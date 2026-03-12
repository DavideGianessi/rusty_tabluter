#[repr(C)]
#[derive(Clone, Copy)]
pub struct Param {
    pub eval: i32,
    pub instab: i32,
}

#[repr(C)]
#[repr(align(64))]
#[derive(Clone, Copy)]
pub struct Weights {
    pub white_piece: Param,
    pub black_piece: Param,
    pub ready: Param,
    pub balance: Param,
    pub first_line: Param,
    pub second_line: Param,
    pub third_line: Param,
    pub solid_control: Param,
    pub black_in: Param,
    pub white_out: Param,
}

impl Weights {
    pub fn new() -> Self {
        Self {
            white_piece: Param {
                eval: -100,
                instab: -10,
            },
            black_piece: Param {
                eval: 100,
                instab: -10,
            },
            ready: Param {
                eval: 80,
                instab: 0,
            },
            balance: Param {
                eval: 50,
                instab: 0,
            },
            first_line: Param {
                eval: 300,
                instab: -50,
            },
            second_line: Param {
                eval: 500,
                instab: -25,
            },
            third_line: Param {
                eval: 700,
                instab: -50,
            },
            solid_control: Param {
                eval: 650,
                instab: -50,
            },
            black_in: Param {
                eval: -10,
                instab: 50,
            },
            white_out: Param {
                eval: -70,
                instab: 80,
            },
        }
    }
}
