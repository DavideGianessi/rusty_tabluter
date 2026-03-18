
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

    pub satisfaction_threshold: i32,
}

impl Weights {
    pub fn new() -> Self {
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
            white_moves: -300,

            satisfaction_threshold: 10000,
        }
    }
}
