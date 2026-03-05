use primitive_types::U256;

pub fn evaluate(state: U256) -> f32 {
    let white_bits: u128 = (state & ((U256::one() << 81) - 1))
        .low_u128();
    let black_bits: u128 = ((state >> 81) & ((U256::one() << 81) - 1))
        .low_u128();
    //let king_row: u32 = ((state >> (2 * 81)) & U256::from(0xF))
    //    .low_u32();
    //let king_col: u32 = ((state >> (2 * 81 + 4)) & U256::from(0xF))
    //    .low_u32();
    let turn: bool = ((state >> (2 * 81 + 8)) & U256::one()) != U256::zero();

    let white_count = white_bits.count_ones() as f32;
    let black_count = black_bits.count_ones() as f32;

    let white_score = white_count;
    let black_score = black_count * 0.75;

    let total = white_score + black_score + 1e-6;

    let eval = (black_score - white_score) / total;

    if !turn {
        eval
    } else {
        -eval
    }
}

pub fn instability_bonus(_state: U256) -> f32 {
    0.0
}
