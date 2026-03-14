use std::collections::HashMap;

use crate::board::{Move, State};
use crate::eval::evaluate;
use crate::weights::Weights;
//use crate::stats::{inc_nodes, inc_tt_hits, reset_stats, print_stats};
//use crate::debug::debug_log;

const DEPTH_PENALTY: i32 = 500;
const MAX_ENERGY: i32 = 2000;

const WIN_SCORE: i32 = 100_000;
const DRAW_SCORE: i32 = -50_000;
const ALPHA_START: i32 = -200_000;
const BETA_START: i32 = 200_000;

pub struct SearchResult {
    pub value: i32,
    pub best_move: Option<Move>,
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub value: i32,
    pub energy: i32,
}

pub struct TranspositionTable {
    pub table: HashMap<u64, TTEntry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::with_capacity(10_000_000),
        }
    }

    pub fn insert(&mut self, key: u64, value: i32, energy: i32) {
        self.table.insert(key, TTEntry { value, energy });
    }

    pub fn get(&self, key: u64, energy: i32) -> Option<i32> {
        if let Some(entry) = self.table.get(&key) {
            if entry.energy >= energy {
                return Some(entry.value);
            }
        }
        None
    }
}

pub fn search(root: State, history: &Vec<u64>, weights: &Weights) -> SearchResult {
    let mut tt = TranspositionTable::new();
    tt.insert(0, -WIN_SCORE, 1_000_000_000);
    tt.insert(1, -WIN_SCORE, 1_000_000_000);
    if root.white_to_move {
        tt.insert(2, -DRAW_SCORE, 1_000_000_000);
        tt.insert(3, DRAW_SCORE, 1_000_000_000);
    } else {
        tt.insert(2, DRAW_SCORE, 1_000_000_000);
        tt.insert(3, -DRAW_SCORE, 1_000_000_000);
    }

    let mut local_history = history.clone();
    let (my_eval, _my_instability) = evaluate(&root, &weights);

    let (value, best_move) = alphabeta(
        root,
        0,
        MAX_ENERGY,
        ALPHA_START,
        BETA_START,
        &mut local_history,
        &mut tt,
        weights,
        -my_eval,
    );

    SearchResult { value, best_move }
}

fn alphabeta(
    state: State,
    depth: usize,
    energy: i32,
    mut alpha: i32,
    beta: i32,
    history: &mut Vec<u64>,
    tt: &mut TranspositionTable,
    weights: &Weights,
    current_eval: i32,
) -> (i32, Option<Move>) {
    let key = state.canonical_hash();

    if let Some(v) = tt.get(key, energy) {
        return (v, None);
    }

    if energy <= 0 {
        return (current_eval, None);
    }

    let mut moves = Vec::with_capacity(128);
    state.generate_moves(&mut moves);

    let mut scored_moves: Vec<(Move, State, u64, i32, i32, bool)> = moves
        .into_iter()
        .map(|mv| {
            let mut child_state = state;
            child_state.apply_move(&mv, &history);
            let child_key = child_state.canonical_hash();

            let capture_bonus = if mv.captured != 0 { 150 } else { 0 };

            let (child_eval, child_instab) = evaluate(&child_state, weights);

            let child_eval = -child_eval;

            let eval_diff = child_eval - current_eval;

            let child_energy =
                energy - DEPTH_PENALTY + child_instab + capture_bonus + (eval_diff / 10);

            let is_tt_hit = tt.table.contains_key(&child_key);

            (
                mv,
                child_state,
                child_key,
                child_eval,
                child_energy,
                is_tt_hit,
            )
        })
        .collect();

    scored_moves.sort_by(|a, b| b.5.cmp(&a.5).then_with(|| b.3.cmp(&a.3)));

    let mut best_value = ALPHA_START - 1;
    let mut best_move = None;

    history.push(state.hash());

    for (mv, child_state, _child_key, child_eval, child_energy, _is_hit) in scored_moves {
        let (mut value, _) = alphabeta(
            child_state,
            depth + 1,
            child_energy,
            -beta,
            -alpha,
            history,
            tt,
            weights,
            -child_eval,
        );

        value = -value;

        if value > best_value {
            best_value = value;
            best_move = Some(mv);
        }

        alpha = alpha.max(value);
        if alpha >= beta {
            break;
        }
    }

    history.pop();

    tt.insert(key, best_value, energy);

    (best_value - best_value.signum(), best_move)
}
