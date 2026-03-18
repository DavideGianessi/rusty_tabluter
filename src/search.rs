use crate::board::{Move, State};
use crate::eval::evaluate;
use crate::weights::Weights;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const WIN_SCORE: i32 = 100_000;
const DRAW_SCORE: i32 = -50_000;
const ALPHA_START: i32 = -200_000;
const BETA_START: i32 = 200_000;

const TT_BITS: usize = 24; 
const TT_SIZE: usize = 1 << TT_BITS; 
const TT_MASK: u64 = (TT_SIZE as u64) - 1;

pub static ABORT_SEARCH: AtomicBool = AtomicBool::new(false);

#[inline(always)]
pub fn should_abort() -> bool {
    ABORT_SEARCH.load(Ordering::Relaxed)
}

#[derive(Default, Clone, Debug)]
pub struct LevelStats {
    pub nodes_visited: u64,
    pub tt_hits: u64,
    pub beta_cutoffs: u64,
    pub satisfaction_cutoffs: u64,
    pub tt_collisions: u64,
}

pub struct SearchResult {
    pub value: i32,
    pub best_move: Option<Move>,
    pub stats: Vec<LevelStats>,
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: u64,
    pub value: i32,
    pub depth_from_root: u32,
}

pub struct TranspositionTable {
    pub table: Box<[TTEntry]>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        let table = vec![TTEntry { hash: 0, value: 0, depth_from_root: 999 }; TT_SIZE].into_boxed_slice();
        Self { table }
    }

    pub fn clear(&mut self) {
        for entry in self.table.iter_mut() {
            entry.hash = 0;
            entry.depth_from_root = 999;
        }
    }

    #[inline(always)]
    pub fn insert(&mut self, key: u64, value: i32, depth: u32, stats: &mut [LevelStats]) {
        let index = (key & TT_MASK) as usize;
        let entry = &mut self.table[index];
        
        if entry.hash == 0 || depth < entry.depth_from_root {
            *entry = TTEntry {
                hash: key,
                value,
                depth_from_root: depth,
            };
        } else if entry.hash != key {
            stats[depth as usize].tt_collisions += 1;
        }
    }

    #[inline(always)]
    pub fn get(&self, key: u64) -> Option<(i32, u32)> {
        let entry = &self.table[(key & TT_MASK) as usize];
        if entry.hash == key {
            Some((entry.value, entry.depth_from_root))
        } else {
            None
        }
    }
}

pub fn search(
    state: &State,
    history: &Vec<u64>,
    weights: &Weights,
    time_limit: Duration,
    debug: bool,
) -> SearchResult {
    let mut tt_prev = TranspositionTable::new();
    let mut tt_curr = TranspositionTable::new();
    let mut last_valid_result = SearchResult {
        value: 0,
        best_move: None,
        stats: vec![LevelStats::default(); 21],
    };

    ABORT_SEARCH.store(false, Ordering::Relaxed);
    let timeout = time_limit.saturating_sub(Duration::from_millis(500));
    thread::spawn(move || {
        thread::sleep(timeout);
        ABORT_SEARCH.store(true, Ordering::Relaxed);
    });

    let start_time = Instant::now();

    let mut prev_score: Option<i32> = None;

    for current_max_depth in 3..=20 {
        if should_abort() { break; }

        tt_curr.clear();
        let mut stats = vec![LevelStats::default(); current_max_depth + 1];
        let mut local_history = history.clone();

        let result = alphabeta(
            *state,
            0,
            current_max_depth,
            ALPHA_START,
            BETA_START,
            &mut local_history,
            &mut tt_curr,
            &tt_prev,
            weights,
            state.white_to_move,
            &mut stats,
            prev_score,
        );

        if let Some((value, mv)) = result {
            last_valid_result = SearchResult { value, best_move: mv, stats };
            prev_score = Some(value);
            
            if debug {
                println!("--- Depth {} Completed in {:?} ---", current_max_depth, start_time.elapsed());
                println!("{:<5} | {:<10} | {:<8} | {:<8} | {:<12} | {:<10}", "Lvl", "Nodes", "TT Hits", "Beta Cutoffs", "Satisfaction Cutoffs", "Collis.");
                println!("{}", "-".repeat(70));
                
                for (lvl, s) in last_valid_result.stats.iter().enumerate() {
                    if s.nodes_visited == 0 { continue; }
                    println!(
                        "{:<5} | {:<10} | {:<8} | {:<8} | {:<12} | {:<10}",
                        lvl, s.nodes_visited, s.tt_hits, s.beta_cutoffs, s.satisfaction_cutoffs, s.tt_collisions
                    );
                }
                println!("Score: {} | Move: {:?}\n", value, mv);
            }

            std::mem::swap(&mut tt_prev, &mut tt_curr);
            if value.abs() >= WIN_SCORE - 100 { break; }
        } else { break; }
    }

    last_valid_result
}

fn alphabeta(
    state: State,
    depth: usize,
    max_depth: usize,
    mut alpha: i32,
    beta: i32,
    history: &mut Vec<u64>,
    tt_curr: &mut TranspositionTable,
    tt_prev: &TranspositionTable,
    weights: &Weights,
    is_white_searcher: bool,
    stats: &mut [LevelStats],
    last_value: Option<i32>,
) -> Option<(i32, Option<Move>)> {
    
    if (depth == 0 || stats[depth].nodes_visited % 65536 == 0) && should_abort() {
        return None;
    }
    stats[depth].nodes_visited += 1;

    if state.win || state.draw {
        if state.win { return Some((-WIN_SCORE, None)); }
        let score = if state.white_to_move == is_white_searcher { DRAW_SCORE } else { -DRAW_SCORE };
        return Some((score, None));
    }

    let key = state.hash();

    if let Some((v, d_entry)) = tt_curr.get(key) {
        if d_entry as usize <= depth {
            stats[depth].tt_hits += 1;
            return Some((v, None));
        }
    }

    if depth >= max_depth {
        let (eval, _) = evaluate(&state, weights);
        return Some((eval, None));
    }

    let mut moves = Vec::with_capacity(128);
    state.generate_moves(&mut moves);
    
    if depth < max_depth.saturating_sub(2) {
        moves.sort_by_cached_key(|mv| {
            let mut child = state;
            child.apply_move(mv, history);
            let child_hash = child.hash();
            if let Some((v, d_entry)) = tt_curr.get(child_hash) {
                if d_entry as usize <= depth + 1 {
                    return 0 - (v / 10); 
                }
            }
            if let Some((v, _)) = tt_prev.get(child_hash) {
                return 100_000 - (v / 10);
            }
            200_000
        });
    }

    let mut best_value = ALPHA_START - 1;
    let mut best_move = None;
    let num_moves = moves.len();

    history.push(key);

    for (i, mv) in moves.into_iter().enumerate() {
        let mut child = state;
        child.apply_move(&mv, history);

        let (mut val, _) = alphabeta(
            child,
            depth + 1,
            max_depth,
            -beta,
            -alpha,
            history,
            tt_curr,
            tt_prev,
            weights,
            is_white_searcher,
            stats,
            last_value.map(|v| -v),
        )?;

        val = -val;

        if val > best_value {
            best_value = val;
            best_move = Some(mv);
        }

        alpha = alpha.max(val);
        
        if alpha >= beta {
            let remaining = (num_moves - (i + 1)) as u64;
            if depth + 1 < stats.len() {
                stats[depth + 1].beta_cutoffs += remaining;
            }
            break;
        }

        if let Some(lv) = last_value {
            let remaining = (num_moves - (i + 1)) as u64;
            if val >= lv + weights.satisfaction_threshold {
                if depth + 1 < stats.len() {
                    stats[depth + 1].satisfaction_cutoffs += remaining;
                }
                break;
            }
        }

    }

    history.pop();

    let final_score = best_value - best_value.signum();
    tt_curr.insert(key, final_score, depth as u32, stats);

    Some((final_score, best_move))
}
