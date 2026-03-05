use std::collections::HashMap;
use primitive_types::U256;
use ahash::AHasher;
use std::hash::{Hash, Hasher};

use crate::debug::debug_log;
use crate::stats::{inc_nodes,inc_tt_hits,reset_stats,print_stats};


use crate::board::{
    generate_moves,
    get_white_win,
    get_black_win,
    get_draw,
};
use crate::eval::{
    evaluate,
    instability_bonus,
};

const DEPTH_PENALTY: f32 = 0.5;
const MAX_ENERGY: f32 = 1.5;

const WIN_SCORE: f32 = 1.0;
const LOSS_SCORE: f32 = -1.0;
const DRAW_SCORE: f32 = 0.0;


pub struct SearchResult {
    pub value: f32,
    pub best_move: Option<U256>,
}


pub fn hash_u64(value: U256) -> u64 {
    let mut hasher = AHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub value: f32,
    pub energy: f32,
}

pub struct TranspositionTable {
    table: HashMap<u64, TTEntry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::with_capacity(1_000_000),
        }
    }

    pub fn insert(&mut self, key: u64, value: f32, energy: f32) {
        self.table.insert(key, TTEntry { value, energy });
    }

    pub fn get(&self, key: u64, energy: f32) -> Option<f32> {
        if let Some(entry) = self.table.get(&key) {
            if entry.energy >= energy {
                return Some(entry.value);
            }
        }
        None
    }
}

pub fn search(root: U256, history: &Vec<U256>) -> SearchResult {
    let mut tt = TranspositionTable::new();
    reset_stats();
    tt.insert(hash_u64(get_white_win()), LOSS_SCORE, f32::INFINITY);
    tt.insert(hash_u64(get_black_win()), LOSS_SCORE, f32::INFINITY);
    tt.insert(hash_u64(get_draw()), DRAW_SCORE, f32::INFINITY);
    let mut local_history = history.clone();
    let (value, best_move) = alphabeta(
        root,
        0,
        MAX_ENERGY,
        -1.5,
        1.5,
        &mut local_history,
        &mut tt,
    );
    print_stats();
    SearchResult {
        value,
        best_move,
    }
}

fn alphabeta(
    state: U256,
    depth: usize,
    energy: f32,
    mut alpha: f32,
    beta: f32,
    history: &mut Vec<U256>,
    tt: &mut TranspositionTable,
) -> (f32, Option<U256>) {
    inc_nodes();
    let key = hash_u64(state);
    debug_log(depth, &format!("state={}",state));
    debug_log(depth, &format!("energy={:.2} alpha={:.2} beta={:.2}",energy,alpha,beta));
    if history.contains(&state) {
        debug_log(depth, "draw by repetition");
        return (DRAW_SCORE, None);
    }
    history.push(state);
    if let Some(v) = tt.get(key, energy) {
        history.pop();
        debug_log(depth, &format!("transposition hit! value:{:.2}",v));
        inc_tt_hits();
        return (v, None);
    }
    if state == get_white_win() {
        history.pop();
        debug_log(depth, "white win");
        return (WIN_SCORE, None);
    }
    if state == get_black_win() {
        history.pop();
        debug_log(depth, "black win");
        return (LOSS_SCORE, None);
    }
    if energy <= 0.0 {
        let v = evaluate(state);
        tt.insert(key, v, energy);
        history.pop();
        debug_log(depth, &format!("out of energy evalution:{:.2}",v));
        return (v, None);
    }
    let moves = generate_moves(state);
    if moves.is_empty() {
        tt.insert(key, LOSS_SCORE, energy);
        history.pop();
        debug_log(depth, "empty moves");
        return (LOSS_SCORE, None);
    }
    let mut scored_moves: Vec<(U256, f32)> = moves
        .into_iter()
        .map(|m| {
            let bonus = instability_bonus(m);
            let child_energy = energy - DEPTH_PENALTY + bonus;
            (m, child_energy)
        })
        .collect();
    scored_moves.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap()
    });
    let mut best_value = -2.0;
    let mut best_move = None;
    for (child, child_energy) in scored_moves {
        let (mut value, _) = alphabeta(
            child,
            depth+1,
            child_energy,
            -beta,
            -alpha,
            history,
            tt,
        );
        value = -value;
        if value > best_value {
            best_value = value;
            best_move = Some(child);
        }
        alpha = alpha.max(value);
        if alpha >= beta {
            debug_log(depth, "alphabeta potatoura");
            break;
        }
    }
    history.pop();
    debug_log(depth, &format!("ricerca children finita, valore:{:.2}",best_value));
    (best_value, best_move)
}
