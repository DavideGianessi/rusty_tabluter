pub struct Param {
    pub eval: i32,
    pub instab: i32,
}

pub struct Weights {
    pub material_pawn: Param,
    pub king_on_throne: Param,
    pub king_near_exit: Param,
    pub shield_pawn: Param,
}

impl Weights {
    pub fn new() -> Self {
        Self {
            material_pawn: Param { eval: 100, instab: 10 }, 
            king_on_throne: Param { eval: -50, instab: 5 },
            king_near_exit: Param { eval: 500, instab: 200 },
            shield_pawn: Param { eval: 30, instab: -10 },
        }
    }
}
