use crate::incremental::State;
use crate::weights::{Weights,Params};



pub fn evaluate(state: &State, w: &Weights) -> (i32, i32) {
    let mut score = 0;
    let mut instab = 0;

    let w_cnt = state.white.count_ones() as i32;
    let b_cnt = state.black.count_ones() as i32;

    score += w_cnt * w.material_pawn.eval;
    instab += w_cnt * w.material_pawn.instab;

    score -= b_cnt * w.material_pawn.eval;
    instab += b_cnt * w.material_pawn.instab;

    let king_near_exit = (state.king & NEAR_EXIT_MASK).count_ones() as i32;
    score += king_near_exit * w.king_near_exit.eval;
    instab += king_near_exit * w.king_near_exit.instab;

    let shield_pieces = (state.white & SHIELD_MASK).count_ones() as i32;
    score += shield_pieces * w.shield_pawn.eval;
    instab += shield_pieces * w.shield_pawn.instab;


    (score, instab)
}
