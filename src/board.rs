use colored::{ColoredString, Colorize};
use primitive_types::U256;

use crate::rot_table::ROT_TABLE;

const DIRS: [(isize, isize); 4] = [
    (-1, 0), // up
    (1, 0),  // down
    (0, -1), // left
    (0, 1),  // right
];

const BOARD_MASK: u128 = (1 << 81) - 1;

#[inline(always)]
pub fn get_white_win() -> U256 {
    U256::from(1u8) << 240
}
#[inline(always)]
pub fn get_black_win() -> U256 {
    (U256::from(1u8) << 241) + (U256::from(1u8) << (2*81+4))
}
#[inline(always)]
pub fn get_draw() -> U256 {
    U256::from(1u8) << 242
}

#[inline(always)]
fn index(row: usize, col: usize) -> usize {
    9 * row + col
}

#[inline(always)]
fn within_bounds(row: isize, col: isize) -> bool {
    row >= 0 && row <= 8 && col >= 0 && col <= 8
}

#[inline(always)]
fn king_row(state: U256) -> usize {
    ((state >> (2 * 81)) & U256::from(15u8)).low_u32() as usize
}

#[inline(always)]
fn king_col(state: U256) -> usize {
    ((state >> (2 * 81 + 4)) & U256::from(15u8)).low_u32() as usize
}

#[inline(always)]
pub fn turn(state: U256) -> bool {
    ((state >> (2 * 81 + 8)) & U256::one()).low_u32() == 1
}

#[inline(always)]
pub fn is_white(state: U256, row: usize, col: usize) -> bool {
    let idx = index(row, col);
    ((state >> idx) & U256::one()) == U256::one()
}

#[inline(always)]
pub fn is_black(state: U256, row: usize, col: usize) -> bool {
    let idx = index(row, col);
    ((state >> (81 + idx)) & U256::one()) == U256::one()
}

#[inline(always)]
pub fn is_king(state: U256, row: usize, col: usize) -> bool {
    row == king_row(state) && col == king_col(state)
}

#[inline(always)]
pub fn is_occupied(state: U256, row: usize, col: usize) -> bool {
    is_white(state, row, col) || is_black(state, row, col)
}

#[inline(always)]
pub fn is_throne(row: usize, col: usize) -> bool {
    row == 4 && col == 4
}

#[inline(always)]
pub fn is_escape(row: usize, col: usize) -> bool {
    (!is_citadel(row, col)) && ((row == 0) || (row == 8) || (col == 0) || (col == 8))
}

#[inline(always)]
pub fn is_citadel(row: usize, col: usize) -> bool {
    ((row == 0 || row == 8) && (col >= 3 && col <= 5))
        || ((row == 1 || row == 7) && (col == 4))
        || ((col == 0 || col == 8) && (row >= 3 && row <= 5))
        || ((col == 1 || col == 7) && (row == 4))
}

#[inline(always)]
pub fn is_killer_surface(row: usize, col: usize) -> bool {
    is_throne(row, col)
        || ((row == 0 || row == 8) && (col == 3 || col == 5))
        || ((row == 1 || row == 7) && (col == 4))
        || ((col == 0 || col == 8) && (row == 3 || row == 5))
        || ((col == 1 || col == 7) && (row == 4))
}

#[inline(always)]
pub fn get_cell(state: U256, row: usize, col: usize) -> char {
    if is_king(state, row, col) {
        return 'K';
    }
    if is_white(state, row, col) {
        return 'W';
    }
    if is_black(state, row, col) {
        return 'B';
    }
    'O'
}

#[inline(always)]
fn get_cell_color(state: U256, row: usize, col: usize) -> ColoredString {
    if is_king(state, row, col) {
        return "██".yellow().bold();
    }
    if is_white(state, row, col) {
        return "██".white();
    }
    if is_black(state, row, col) {
        return "██".red();
    }
    if is_throne(row, col) {
        return "██".green();
    }
    if is_citadel(row, col) {
        return "██".bright_black();
    }
    if is_escape(row, col) {
        return "██".bright_blue();
    }
    "██".black()
}

#[inline(always)]
fn white_bitboard(state: U256) -> u128 {
    (state & U256::from(BOARD_MASK)).low_u128()
}

#[inline(always)]
fn black_bitboard(state: U256) -> u128 {
    ((state >> 81) & U256::from(BOARD_MASK)).low_u128()
}

pub fn canonize(state: U256) -> U256 {
    if state == get_black_win() || state == get_white_win() || state == get_draw() {
        return state;
    }
    let mut whiteout: [u128; 8] = [0; 8];
    for i in 0..9 {
        let row: u128 = ((state >> (9 * i)) & U256::from(511u16)).low_u128();

        let rot: u128 = ROT_TABLE[row as usize];

        let rev_row: u128 = row.reverse_bits() >> (128 - 9);
        let rev_rot: u128 = ROT_TABLE[rev_row as usize];

        whiteout[0] |= rot << i;
        whiteout[1] |= rot << (8 - i);
        whiteout[2] |= rev_rot << i;
        whiteout[3] |= rev_rot << (8 - i);
        whiteout[4] |= row << (9 * i);
        whiteout[5] |= row << (9 * (8 - i));
        whiteout[6] |= rev_row << (9 * i);
        whiteout[7] |= rev_row << (9 * (8 - i));
    }

    let mut blackout: [u128; 8] = [0; 8];
    for i in 0..9 {
        let row: u128 = ((state >> (81 + (9 * i))) & U256::from(511u16)).low_u128();

        let rot: u128 = ROT_TABLE[row as usize];

        let rev_row: u128 = row.reverse_bits() >> (128 - 9);
        let rev_rot: u128 = ROT_TABLE[rev_row as usize];

        blackout[0] |= rot << i;
        blackout[1] |= rot << (8 - i);
        blackout[2] |= rev_rot << i;
        blackout[3] |= rev_rot << (8 - i);
        blackout[4] |= row << (9 * i);
        blackout[5] |= row << (9 * (8 - i));
        blackout[6] |= rev_row << (9 * i);
        blackout[7] |= rev_row << (9 * (8 - i));
    }

    let kingrow: u32 = ((state >> (2 * 81)) & U256::from(15u8)).low_u32();

    let kingcol: u32 = ((state >> (2 * 81 + 4)) & U256::from(15u8)).low_u32();

    let mut kingpos: [u32; 8] = [0; 8];

    kingpos[0] = (kingrow << 4) | kingcol;
    kingpos[1] = ((8 - kingrow) << 4) | kingcol;
    kingpos[2] = (kingrow << 4) | (8 - kingcol);
    kingpos[3] = ((8 - kingrow) << 4) | (8 - kingcol);
    kingpos[4] = (kingcol << 4) | kingrow;
    kingpos[5] = (kingcol << 4) | (8 - kingrow);
    kingpos[6] = ((8 - kingcol) << 4) | kingrow;
    kingpos[7] = ((8 - kingcol) << 4) | (8 - kingrow);

    let turn: u32 = (state >> (2 * 81 + 8) & U256::one()).low_u32();

    let mut canonical: U256 = U256::from(whiteout[0])
        | (U256::from(blackout[0]) << 81)
        | (U256::from(kingpos[0]) << (2 * 81))
        | (U256::from(turn) << (2 * 81 + 8));


    for i in 1..8 {
        let challenger: U256 = U256::from(whiteout[i])
            | (U256::from(blackout[i]) << 81)
            | (U256::from(kingpos[i]) << (2 * 81))
            | (U256::from(turn) << (2 * 81 + 8));

        canonical = canonical.min(challenger);
    }

    canonical
}

pub fn parse_position(input: &str) -> U256 {
    let mut state = U256::zero();
    let mut k_row = 0usize;
    let mut k_col = 0usize;

    let lines: Vec<&str> = input.lines().collect();
    assert!(lines.len() >= 10);

    for row in 0..9 {
        let chars: Vec<char> = lines[row].chars().collect();
        assert!(chars.len() == 9);

        for col in 0..9 {
            let idx = index(row, col);

            match chars[col] {
                'W' => state |= U256::one() << idx,
                'B' => state |= U256::one() << (81 + idx),
                'K' => {
                    state |= U256::one() << idx;
                    k_row = row;
                    k_col = col;
                }
                'O' => {}
                _ => panic!("Carattere non valido"),
            }
        }
    }

    state |= U256::from(k_row as u32) << (2 * 81);
    state |= U256::from(k_col as u32) << (2 * 81 + 4);

    if lines[9].trim() == "turn: W" {
        state |= U256::one() << (2 * 81 + 8);
    }

    state
}

#[allow(dead_code)]
pub fn format_position(state: U256) -> String {
    if state == get_black_win() {
        return "BLACK HAS WON THE GAME".to_string();
    }
    if state == get_white_win() {
        return "WHITE HAS WON THE GAME".to_string();
    }
    if state == get_draw() {
        return "IT'S A DRAW".to_string();
    }
    let mut output = String::new();

    for row in 0..9 {
        for col in 0..9 {
            output.push(get_cell(state, row, col));
        }
        output.push('\n');
    }

    output.push_str(if turn(state) { "turn: W" } else { "turn: B" });

    output
}

#[allow(dead_code)]
pub fn print_board(state: U256) {
    if state == get_black_win() {
        println!("BLACK HAS WON THE GAME");
        return;
    }
    if state == get_white_win() {
        println!("WHITE HAS WON THE GAME");
        return;
    }
    if state == get_draw() {
        println!("IT'S A DRAW");
        return;
    }
    println!();
    println!("TURN: {}", if turn(state) { "WHITE" } else { "BLACK" });
    println!();

    println!("+----+----+----+----+----+----+----+----+----+");

    for row in 0..9 {
        print!("|");

        for col in 0..9 {
            let cell = get_cell_color(state, row, col);
            print!(" {} |", cell);
        }

        println!();
        println!("+----+----+----+----+----+----+----+----+----+");
    }
}

pub fn get_printable_board(state: U256) -> String {
    if state == get_black_win() {
        return "BLACK HAS WON THE GAME\r\n".to_string();
    }
    if state == get_white_win() {
        return "WHITE HAS WON THE GAME\r\n".to_string();
    }
    if state == get_draw() {
        return "IT'S A DRAW\r\n".to_string();
    }
    let mut output = String::new();
    output.push_str("\r\n");
    output.push_str(&format!(
        "TURN: {}\r\n\n",
        if turn(state) { "WHITE" } else { "BLACK" }
    ));
    output.push_str("+----+----+----+----+----+----+----+----+----+\r\n");
    for row in 0..9 {
        output.push('|');

        for col in 0..9 {
            let cell = get_cell_color(state, row, col);
            // convert ColoredString → String
            output.push_str(&format!(" {} |", cell));
        }
        output.push_str("\r\n");
        output.push_str("+----+----+----+----+----+----+----+----+----+\r\n");
    }
    output
}

#[inline(always)]
fn pop_piece(pieces: &mut u128) -> Option<usize> {
    if *pieces == 0 {
        return None;
    }
    let tz = pieces.trailing_zeros() as usize;
    *pieces &= *pieces - 1;
    Some(tz)
}

pub fn generate_moves(state: U256, history: &Vec<U256>) -> Vec<U256> {
    let mut moves: Vec<U256> = vec![];
    let mut moves_canonized: Vec<U256> = vec![];
    let mut pieces = if turn(state) {
        white_bitboard(state)
    } else {
        black_bitboard(state)
    };
    while let Some(idx) = pop_piece(&mut pieces) {
        let row = idx / 9;
        let col = idx % 9;
        let can_enter_citadels = (!turn(state)) && is_citadel(row, col);
        for (dr, dc) in DIRS {
            let mut r = row as isize + dr;
            let mut c = col as isize + dc;

            while r >= 0 && r < 9 && c >= 0 && c < 9 {
                let ur = r as usize;
                let uc = c as usize;

                if is_occupied(state, ur, uc)
                    || is_throne(ur, uc)
                    || (is_citadel(ur, uc) && !can_enter_citadels)
                {
                    break;
                }
                let mut new_state = apply_move(state, row, col, ur, uc);
                if history.contains(&new_state){
                    new_state = get_draw();
                }
                if !moves_canonized.contains(&canonize(new_state)) {
                    moves.push(new_state);
                    moves_canonized.push(canonize(new_state));
                }

                r += dr;
                c += dc;
            }
        }
    }
    moves
}


#[inline(always)]
pub fn has_legal_moves(state: U256) -> bool {
    let mut pieces = if turn(state) {
        white_bitboard(state)
    } else {
        black_bitboard(state)
    };
    const DIRS: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    while let Some(idx) = pop_piece(&mut pieces) {
        let row = idx / 9;
        let col = idx % 9;
        let can_enter_citadels = (!turn(state)) && is_citadel(row, col);
        for (dr, dc) in DIRS {
            let r = row as isize + dr;
            let c = col as isize + dc;
            while r >= 0 && r < 9 && c >= 0 && c < 9 {
                let ur = r as usize;
                let uc = c as usize;
                if is_occupied(state, ur, uc)
                    || is_throne(ur, uc)
                    || (is_citadel(ur, uc) && !can_enter_citadels)
                {
                    break;
                }
                return true;
            }
        }
    }
    false
}

#[inline(always)]
fn empty_cell(state: &mut U256, row: usize, col: usize) {
    *state &= !(U256::from(1) << (index(row, col)));
    *state &= !(U256::from(1) << (81 + index(row, col)));
}
#[inline(always)]
fn set_white(state: &mut U256, row: usize, col: usize) {
    *state |= U256::from(1) << (index(row, col));
}
#[inline(always)]
fn set_black(state: &mut U256, row: usize, col: usize) {
    *state |= U256::from(1) << (81 + index(row, col));
}
#[inline(always)]
fn is_opposite(state: U256, row: usize, col: usize) -> bool {
    (is_black(state, row, col) && turn(state)) || (is_white(state, row, col) && !turn(state))
}
#[inline(always)]
fn is_same(state: U256, row: usize, col: usize) -> bool {
    (is_black(state, row, col) && !turn(state)) || (is_white(state, row, col) && turn(state))
}
#[inline(always)]
fn is_special_king_situation(state: U256, row: usize, col: usize) -> bool {
    is_king(state, row, col)
        && ((row == 4 && (col >= 3 && col <= 5)) || (col == 4 && (row >= 3 && row <= 5)))
}
#[inline(always)]
fn is_special_king_capture(state: U256, row: usize, col: usize) -> bool {
    (is_black(state, row - 1, col) || is_throne(row - 1, col))
        && (is_black(state, row + 1, col) || is_throne(row + 1, col))
        && (is_black(state, row, col - 1) || is_throne(row, col - 1))
        && (is_black(state, row, col + 1) || is_throne(row, col + 1))
}
#[inline(always)]
fn move_king(state: &mut U256, row: usize, col: usize) {
    let king_mask: U256 =
        !((U256::from(0b1111u8) << (2 * 81)) | (U256::from(0b1111u8) << (2 * 81 + 4)));
    *state &= king_mask;
    *state |= U256::from(row as u32) << (2 * 81);
    *state |= U256::from(col as u32) << (2 * 81 + 4);
}
#[inline(always)]
fn toggle_turn(state: &mut U256) {
    let turn_bit: U256 = U256::from(1) << (2 * 81 + 8);
    *state ^= turn_bit;
}

fn apply_move(
    state: U256,
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
) -> U256 {
    let mut state = state;
    if turn(state) {
        set_white(&mut state, end_row, end_col);
        if is_king(state, start_row, start_col) {
            move_king(&mut state, end_row, end_col);
            if is_escape(end_row, end_col) {
                return get_white_win();
            }
        }
    } else {
        set_black(&mut state, end_row, end_col);
    }
    empty_cell(&mut state, start_row, start_col);
    for (dr, dc) in DIRS {
        let capture_row = end_row as isize + dr;
        let capture_col = end_col as isize + dc;
        let capturing_row = end_row as isize + 2 * dr;
        let capturing_col = end_col as isize + 2 * dc;
        if within_bounds(capturing_row, capturing_col) {
            let capture_row = capture_row as usize;
            let capture_col = capture_col as usize;
            let capturing_row = capturing_row as usize;
            let capturing_col = capturing_col as usize;
            if is_opposite(state, capture_row, capture_col) {
                if is_special_king_situation(state, capture_row, capture_col) {
                    if is_special_king_capture(state, capture_row, capture_col) {
                        return get_black_win();
                    }
                } else {
                    if is_same(state, capturing_row, capturing_col)
                        || is_killer_surface(capturing_row, capturing_col)
                    {
                        if is_king(state, capture_row, capture_col) {
                            return get_black_win();
                        } else {
                            empty_cell(&mut state, capture_row, capture_col);
                        }
                    }
                }
            }
        }
    }
    toggle_turn(&mut state);
    if !has_legal_moves(state) {
        if turn(state) {
            state = get_black_win();
        } else {
            state = get_white_win();
        }
    }
    state
}

pub fn extract_move(
    before: U256,
    after: U256,
    history: &Vec<U256>,
) -> Option<(usize, usize, usize, usize)> {
    for from in 0..81 {
        for to in 0..81 {
            let sr = from / 9;
            let sc = from % 9;
            let er = to / 9;
            let ec = to % 9;
            if check_move(before, sr,sc,er,ec) {
                let mut candidate = apply_move(before, sr, sc, er, ec);
                if history.contains(&candidate){
                    candidate = get_draw();
                }
                if candidate == after {
                    return Some((sr,sc,er,ec));
                }
            }
        }
    }
    None
}

pub fn check_move(state: U256, sr:usize, sc:usize, er:usize, ec:usize) -> bool {
    if sr>=9 || sc>=9 || er>=9 || ec>=9 {
        return false;
    }
    if !((sr==er) ^ (sc==ec)) {
        return false;
    }
    let is_white_turn = turn(state);
    if !((is_white(state,sr,sc) && is_white_turn) || (is_black(state,sr,sc) && !is_white_turn)) {
        return false;
    }
    let can_enter_citadels = (!turn(state)) && is_citadel(sr, sc);
    let dr: isize = (er as isize - sr as isize).signum();
    let dc: isize = (ec as isize - sc as isize).signum();
    let mut r = sr;
    let mut c = sc;
    while r !=  er || c != ec {
        r = (r as isize + dr) as usize;
        c = (c as isize + dc) as usize;
        if is_occupied(state, r, c)
            || is_throne(r, c)
            || (is_citadel(r, c) && !can_enter_citadels)
        {
            return false;
        }
    }
    return true;
}
