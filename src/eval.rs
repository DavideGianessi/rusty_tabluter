use crate::board::State;
use crate::weights::Weights;

const THRONE: u128 = 1 << 9 * 4 + 4;
const CITADELS: [u128; 4] = [
    8248,
    35390664736768,
    9033621893545984,
    264600096993289808379904,
];
const ALL_CITADELS: u128 = 264600106062302366670904;
const QUADRANTS: [u128; 4] = [
    2384656163722786949300224,
    297795283697879662723072,
    35392547733055,
    9033652084400632,
];
const WHITE_QUADRANTS: [u128; 4] = [
    18926431620092124741151,
    302822905921471986524656,
    146688491952586221422596,
    2346877790876804363354176,
];
const EXTERNAL: u128 = 2152881580533779429035975;
const INTERNAL: u128 = 290208130212560896;

const FIRST_LINE: [u128; 4] = [
    526336,
    33587200,
    36929516944438067200,
    592601653367919345664,
];
const FIRST_INFILTRATION: [u128; 4] = [
    263687,
    67305920,
    33084253510596590305280,
    2119166570873771717033984,
];
const SECOND_LINE: [u128; 4] = [1048576, 16777216, 72057594037927936, 1152921504606846976];
const SECOND_INFILTRATION: [u128; 4] = [
    790023,
    100893120,
    33121183027541028372480,
    2119759172527139636379648,
];
const THIRD_LINE: [u128; 4] = [
    538968064,
    8598323200,
    144255925564211200,
    578712552117108736,
];
const THIRD_INFILTRATION: [u128; 4] = [
    270278151,
    17297555904,
    33195042131798648684544,
    2120055477857423223422976,
];
const SOLID_CONTROL: [u128; 4] = [
    540016640,
    8615100416,
    216313519602139136,
    1731634056723955712,
];

pub fn evaluate(state: &State, w: &Weights) -> i32 {

    let white_count = state.white.count_ones() as i32;
    let black_count = state.black.count_ones() as i32;

    let ready = {
        let c0 = (state.black & CITADELS[0]).count_ones();
        let c1 = (state.black & CITADELS[1]).count_ones();
        let c2 = (state.black & CITADELS[2]).count_ones();
        let c3 = (state.black & CITADELS[3]).count_ones();
        c0.min(c1).min(c2).min(c3) as i32
    };

    let balance = {
        let c0 = (state.black & QUADRANTS[0]).count_ones();
        let c1 = (state.black & QUADRANTS[1]).count_ones();
        let c2 = (state.black & QUADRANTS[2]).count_ones();
        let c3 = (state.black & QUADRANTS[3]).count_ones();
        c0.min(c1).min(c2).min(c3) as i32
    };

    let mut first_line = 0;
    let mut second_line = 0;
    let mut third_line = 0;
    let mut solid_control = 0;

    for i in 0..4 {
        let first = (state.black & FIRST_LINE[i] == FIRST_LINE[i]
            && state.white & FIRST_INFILTRATION[i] == 0) as i32;
        let second = (state.black & SECOND_LINE[i] == SECOND_LINE[i]
            && state.white & SECOND_INFILTRATION[i] == 0) as i32;
        let third = (state.black & THIRD_LINE[i] == THIRD_LINE[i]
            && state.white & THIRD_INFILTRATION[i] == 0) as i32;
        let solid = (state.black & SOLID_CONTROL[i] == SOLID_CONTROL[i]) as i32;
        third_line += third;
        solid_control += solid & (1 - third);
        second_line += second & (1 - solid) & (1 - third);
        first_line += first & (1 - second) & (1 - solid) & (1 - third);
    }

    let black_in = (state.black & INTERNAL).count_ones() as i32;
    let white_out = (state.white & EXTERNAL).count_ones() as i32;

    let white_moves = state.white_mobility();

    let mut distance_from_unblocked:i32=15;
    for i in 0..4 {
        if !(state.black & FIRST_LINE[i] == FIRST_LINE[i] 
            || state.black & SECOND_LINE[i] == SECOND_LINE[i] 
            || state.black & THIRD_LINE[i] == THIRD_LINE[i]
            || state.black & SOLID_CONTROL[i] == SOLID_CONTROL[i]){
            let posx = (9*(i%2)) as i32;
            let posy = (9*(i/2)) as i32;
            let king_idx = state.king.trailing_zeros() as i32;
            let king_row = (king_idx / 9) as i32;
            let king_col = (king_idx % 9) as i32;
            let dist = (posy-king_row).abs() + (posx-king_col).abs();
            distance_from_unblocked=distance_from_unblocked.min(dist);
        }
    }

    let mut quadrant_advantage:i32 = -1;
    for i in 0..4 {
        if !(state.black & FIRST_LINE[i] == FIRST_LINE[i] 
            || state.black & SECOND_LINE[i] == SECOND_LINE[i] 
            || state.black & THIRD_LINE[i] == THIRD_LINE[i]
            || state.black & SOLID_CONTROL[i] == SOLID_CONTROL[i]){
            let advantage:i32 = (state.white & WHITE_QUADRANTS[i]).count_ones() as i32 - (state.black & WHITE_QUADRANTS[i]).count_ones() as i32;
            quadrant_advantage = quadrant_advantage.max(advantage);
        }
    }

    let adj_mask=(state.king >> 1) | (state.king << 1) | (state.king >> 9) | (state.king << 9);
    let encirclement = (adj_mask & (state.black | ALL_CITADELS)).count_ones() as i32;

    let mut square_formation=false;

    if white_count>4 { 
        let pos = state.king;
        let squares = [
            pos | (pos << 1) | (pos << 9) | (pos << 10), // pos = top-left
            pos | (pos >> 1) | (pos << 9) | (pos << 8),  // pos = top-right
            pos | (pos << 1) | (pos >> 9) | (pos >> 8),  // pos = bottom-left
            pos | (pos >> 1) | (pos >> 9) | (pos >> 10), // pos = bottom-right
        ];

        for square in squares {
            if (square & state.white == square) && (state.king != THRONE) {
                square_formation = true;
            }
        }
    }


    let mut score = white_count * w.white_piece
        + black_count * w.black_piece
        + ready * w.ready
        + balance * w.balance
        + first_line * w.first_line
        + second_line * w.second_line
        + third_line * w.third_line
        + solid_control * w.solid_control
        + black_in * w.black_in
        + white_out * w.white_out
        + white_moves * w.white_moves
        + distance_from_unblocked * w.distance_from_unblocked
        + quadrant_advantage * w.quadrant_advantage
        + encirclement * w.encirclement;

    if (square_formation) && (w.square_formation==1) && (score> 100) {
        score = (100-5*white_count+3*black_count).max(1) as i32;
    }

    let turnflip = (state.white_to_move as i32 * -2) + 1;
    score *= turnflip;

    score
}
