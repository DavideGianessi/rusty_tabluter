
#[repr(C)]
#[repr(align(64))]
#[derive(Clone, Copy)]
pub struct Weights {
    pub white_piece: i32,
    pub black_piece: i32,
    pub ready: i32,
    pub balance: i32,
    pub first_line: i32,
    pub second_line: i32,
    pub third_line: i32,
    pub solid_control: i32,
    pub black_in: i32,
    pub white_out: i32,
    pub white_moves: i32,

    pub distance_from_unblocked: i32,
    pub quadrant_advantage: i32,
    pub encirclement: i32,
    pub square_formation: i32,

    pub satisfaction_threshold: i32,
}

impl Weights {
    pub fn new(is_white: bool) -> Self {
        if is_white {
            Self {
                white_piece: -200,
                black_piece: 150,
                ready: 0,
                balance: 0,
                first_line: 0,
                second_line: 0,
                third_line: 0,
                solid_control: 0,
                black_in: 0,
                white_out: 0,
                white_moves: -5,

                distance_from_unblocked: 4,
                quadrant_advantage: -10,
                encirclement: 150,
                square_formation: 1,

                satisfaction_threshold: 1000000,
            }
        } else {
            Self {
                white_piece: -100,
                black_piece: 100,
                ready: 20,
                balance: 5,
                first_line: 400,
                second_line: 600,
                third_line: 800,
                solid_control: 750,
                black_in: 0,
                white_out: -40,
                white_moves: -400,

                distance_from_unblocked: 20,
                quadrant_advantage: 0,
                encirclement: 50,
                square_formation: 0,

                satisfaction_threshold: 1000000,
            }
        }
    }
}
