use primitive_types::U256;

use crate::zobrist_keys::{Z_BLACK, Z_KING, Z_TURN, Z_WHITE};

const THRONE: u128 = 1 << 9 * 4 + 4;
const CITADELS: u128 = 264600106062302366670904;
const HALF_CITADELS_1: u128 = 9033621893554232;
const HALF_CITADELS_2: u128 = 264600097028680473116672;
const KILLER: u128 = 189042242319826649358376;
const SPECIAL_KING_ZONE: u128 = 566800391602176;
const ALIVE_KING_ZONE: u128 = 2347321108901937150976;

#[inline(always)]
fn extract_bit(bitboard: u128, idx: i8) -> bool {
    let mask = if idx < 0 { 0 } else { 1 };
    ((bitboard.wrapping_shr(idx as u32)) & mask) != 0
}

#[inline(always)]
fn extract_bit_inplace(bitboard: u128, idx: i8) -> u128 {
    let bitmask = 1u128.wrapping_shl(idx as u32);
    let range_mask = if idx < 0 { 0 } else { !0u128 };
    bitboard & bitmask & range_mask
}

#[inline(always)]
fn bit_to_mask(cond: bool) -> u128 {
    -(cond as i128) as u128
}

#[inline(always)]
fn pop_piece(pieces: &mut u128) -> Option<u8> {
    if *pieces == 0 {
        return None;
    }
    let tz = pieces.trailing_zeros() as u8;
    *pieces &= *pieces - 1;
    Some(tz)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub white: u128,
    pub black: u128,
    pub king: u128,

    pub white_to_move: bool,

    pub hash: [u64; 8],
}

#[derive(Clone, Copy)]
pub struct Move {
    pub fr: u8,
    pub fc: u8,
    pub tr: u8,
    pub tc: u8,

    pub captured: u128,
}

impl State {
    pub fn new() -> Self {
        let pos_str = "\
            OOOBBBOOO\n\
            OOOOBOOOO\n\
            OOOOWOOOO\n\
            BOOOWOOOB\n\
            BBWWKWWBB\n\
            BOOOWOOOB\n\
            OOOOWOOOO\n\
            OOOOBOOOO\n\
            OOOBBBOOO\n\
            turn: W";
        Self::from_position_string(pos_str)
    }

    #[allow(dead_code)]
    pub fn from_u256(pos: U256) -> Self {
        let board_mask = (U256::one() << 81) - U256::one();

        let white = (pos & board_mask).low_u128();
        let black = ((pos >> 81) & board_mask).low_u128();

        let krow = ((pos >> 162) & U256::from(0xF)).low_u128() as u8;
        let kcol = ((pos >> 166) & U256::from(0xF)).low_u128() as u8;
        let king = 1u128 << (9 * krow + kcol);

        let white_to_move = (pos >> 170).low_u128() & 1 != 0;

        let mut s = State {
            white,
            black,
            king,
            white_to_move,
            hash: [0; 8],
        };
        s.compute_full_hash();
        s
    }

    pub fn is_black(&self, row: u8, col: u8) -> bool {
        ((self.black >> (row * 9 + col)) & 1) != 0
    }
    pub fn is_white(&self, row: u8, col: u8) -> bool {
        ((self.white >> (row * 9 + col)) & 1) != 0
    }
    pub fn is_king(&self, row: u8, col: u8) -> bool {
        ((self.king >> (row * 9 + col)) & 1) != 0
    }

    #[allow(dead_code)]
    pub fn to_u256(&self) -> U256 {
        let (krow, kcol) = self.king_position();
        U256::from(self.white)
            | (U256::from(self.black) << 81)
            | (U256::from(krow) << (2 * 81))
            | (U256::from(kcol) << (2 * 81 + 4))
            | (U256::from(self.white_to_move as u8) << (2 * 81 + 8))
    }

    pub fn from_position_string(pos: &str) -> Self {
        let mut white = 0u128;
        let mut black = 0u128;
        let mut king = 0u128;
        let mut white_to_move = false;

        let lines: Vec<&str> = pos.lines().collect();
        for row in 0..9 {
            let chars: Vec<char> = lines[row].chars().collect();
            for col in 0..9 {
                let bit = 1u128 << (9 * row + col);
                match chars[col] {
                    'W' => white |= bit,
                    'B' => black |= bit,
                    'K' => {
                        white |= bit;
                        king = bit;
                    }
                    _ => {}
                }
            }
        }

        if lines.len() > 9 && lines[9].contains("turn: W") {
            white_to_move = true;
        }

        let mut s = State {
            white,
            black,
            king,
            white_to_move,
            hash: [0; 8],
        };
        s.compute_full_hash();
        s
    }

    #[allow(dead_code)]
    pub fn to_position_string(&self) -> String {
        if self.king == 3 {
            if self.hash[0] % 2 == 1 {
                return "WHITE HAS WON THE GAME".to_string();
            }
            if self.hash[0] >= 2 {
                return "IT'S A DRAW".to_string();
            }
            return "BLACK HAS WON THE GAME".to_string();
        }

        let mut output = String::new();
        for r in 0..9 {
            for c in 0..9 {
                let bit = 1u128 << (9 * r + c);
                if (self.king & bit) != 0 {
                    output.push('K');
                } else if (self.white & bit) != 0 {
                    output.push('W');
                } else if (self.black & bit) != 0 {
                    output.push('B');
                } else {
                    output.push('O');
                }
            }
            output.push('\n');
        }
        output.push_str(if self.white_to_move {
            "turn: W"
        } else {
            "turn: B"
        });
        output
    }

    pub fn set_win(&mut self) {
        for i in 0..8 {
            self.hash[i] = 0 + self.white_to_move as u64;
        }
    }

    pub fn set_draw(&mut self) {
        for i in 0..8 {
            self.hash[i] = 2 + self.white_to_move as u64;
        }
    }

    pub fn king_position(&self) -> (u8, u8) {
        let index = self.king.trailing_zeros() as u8;
        let row = index / 9;
        let col = index % 9;
        (row, col)
    }

    fn piece_has_moves(&self, r: u8, c: u8) -> bool {
        let idx = (9 * r + c) as i8;
        let mut occupied = self.white | self.black | THRONE;
        if ((1 << (r * 9 + c)) & CITADELS) == 0 {
            occupied = occupied | CITADELS;
        }
        let occupied = occupied;
        extract_bit(!occupied, idx + 9)
            || extract_bit(!occupied, idx - 9)
            || extract_bit(!occupied, idx + 1)
            || extract_bit(!occupied, idx - 1)
    }

    fn generate_piece_moves(&self, r: u8, c: u8, moves: &mut Vec<Move>) {
        let directions: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        let mut occupied = self.white | self.black | THRONE;
        if ((1 << (r * 9 + c)) & HALF_CITADELS_1) == 0 {
            occupied = occupied | HALF_CITADELS_1;
        }
        if ((1 << (r * 9 + c)) & HALF_CITADELS_2) == 0 {
            occupied = occupied | HALF_CITADELS_2;
        }
        let occupied = occupied;

        for &(dr, dc) in &directions {
            let mut curr_r = r as i8 + dr;
            let mut curr_c = c as i8 + dc;

            while curr_r >= 0 && curr_r < 9 && curr_c >= 0 && curr_c < 9 {
                let tr = curr_r as u8;
                let tc = curr_c as u8;
                let bit = 1u128 << (9 * tr + tc);

                if (occupied & bit) != 0 {
                    break;
                }

                let captured = self.compute_captures(tr, tc);
                moves.push(Move {
                    fr: r,
                    fc: c,
                    tr,
                    tc,
                    captured,
                });
                curr_r += dr;
                curr_c += dc;
            }
        }
    }

    pub fn generate_moves(&self, moves: &mut Vec<Move>) {
        let mut pieces = if self.white_to_move {
            self.white
        } else {
            self.black
        };
        while let Some(idx) = pop_piece(&mut pieces) {
            let row = idx / 9;
            let col = idx % 9;
            self.generate_piece_moves(row, col, moves);
        }
    }

    pub fn has_moves(&self) -> bool {
        let mut pieces = if self.white_to_move {
            self.white
        } else {
            self.black
        };
        while let Some(idx) = pop_piece(&mut pieces) {
            let row = idx / 9;
            let col = idx % 9;
            if self.piece_has_moves(row, col) {
                return true;
            }
        }
        false
    }

    fn compute_captures(&self, tr: u8, tc: u8) -> u128 {
        let tr = tr as i8;
        let tc = tc as i8;
        let mut res = 0u128;

        let special_king = self.king & SPECIAL_KING_ZONE & bit_to_mask(!self.white_to_move);
        let opposite = (bit_to_mask(self.white_to_move) & self.black)
            | ((bit_to_mask(!self.white_to_move) & self.white) ^ special_king);
        let flankers = (bit_to_mask(self.white_to_move) & self.white)
            | (bit_to_mask(!self.white_to_move) & self.black)
            | KILLER;
        let new_flankers = flankers | (1u128 << (tr * 9 + tc));

        let directions: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dr, dc) in directions {
            let capr: i8 = tr + dr;
            let capc: i8 = tc + dc;
            let flankr: i8 = tr + 2 * dr;
            let flankc: i8 = tc + 2 * dc;

            let cap_idx = capr * 9 + capc;
            let flank_idx = flankr * 9 + flankc;

            let bounds = 0 <= flankr && flankr <= 8 && 0 <= flankc && flankc <= 8;
            let flanked = extract_bit(new_flankers, flank_idx);
            res |= extract_bit_inplace(opposite, cap_idx) & bit_to_mask(bounds && flanked);

            let killing_king = extract_bit(new_flankers, cap_idx + 9)
                && extract_bit(new_flankers, cap_idx - 9)
                && extract_bit(new_flankers, cap_idx + 1)
                && extract_bit(new_flankers, cap_idx - 1);
            res |= extract_bit_inplace(special_king, cap_idx) & bit_to_mask(killing_king);
        }
        res
    }

    pub fn compute_full_hash(&mut self) {
        for i in 0..8 {
            let mut h = 0u64;
            if self.white_to_move {
                h ^= Z_TURN;
            }

            let mut w = self.white;
            while w != 0 {
                let idx = w.trailing_zeros() as usize;
                h ^= Z_WHITE[i][idx];
                w &= w - 1;
            }
            let mut b = self.black;
            while b != 0 {
                let idx = b.trailing_zeros() as usize;
                h ^= Z_BLACK[i][idx];
                b &= b - 1;
            }
            let mut k = self.king;
            while k != 0 {
                let idx = k.trailing_zeros() as usize;
                h ^= Z_KING[i][idx];
                k &= k - 1;
            }

            self.hash[i] = h;
        }
    }

    pub fn update_hash(&mut self, mv: &Move) {
        let fr_idx = (mv.fr * 9 + mv.fc) as usize;
        let tr_idx = (mv.tr * 9 + mv.tc) as usize;

        let is_king = (self.king & (1u128 << fr_idx)) != 0;
        let is_white = (self.white & (1u128 << fr_idx)) != 0;

        for i in 0..8 {
            let mut h = self.hash[i];
            if is_king {
                h ^= Z_KING[i][fr_idx] ^ Z_KING[i][tr_idx];
            } else if is_white {
                h ^= Z_WHITE[i][fr_idx] ^ Z_WHITE[i][tr_idx];
            } else {
                h ^= Z_BLACK[i][fr_idx] ^ Z_BLACK[i][tr_idx];
            }

            let mut caps = mv.captured;
            while caps != 0 {
                let cap_idx = caps.trailing_zeros() as usize;
                let cap_bit = 1u128 << cap_idx;

                if (self.king & cap_bit) != 0 {
                    h ^= Z_KING[i][cap_idx];
                } else if (self.white & cap_bit) != 0 {
                    h ^= Z_WHITE[i][cap_idx];
                } else if (self.black & cap_bit) != 0 {
                    h ^= Z_BLACK[i][cap_idx];
                }

                caps &= caps - 1;
            }
            h ^= Z_TURN;
            self.hash[i] = h;
        }
    }

    pub fn canonical_hash(&self) -> u64 {
        *self.hash.iter().min().unwrap()
    }

    pub fn hash(&self) -> u64 {
        self.hash[0]
    }

    pub fn apply_move(&mut self, mv: &Move, history: &Vec<u64>) {
        self.update_hash(mv);

        let fr_idx: i8 = (mv.fr * 9 + mv.fc) as i8;
        let to_idx: i8 = (mv.tr * 9 + mv.tc) as i8;
        let white_pawn = extract_bit(self.white, fr_idx) as u128;
        self.white |= white_pawn << to_idx;
        self.white &= !(1 << fr_idx);
        let black_pawn = extract_bit(self.black, fr_idx) as u128;
        self.black |= black_pawn << to_idx;
        self.black &= !(1 << fr_idx);
        let king_pawn = extract_bit(self.king, fr_idx) as u128;
        self.king |= king_pawn << to_idx;
        self.king &= !(1 << fr_idx);

        self.white &= !mv.captured;
        self.black &= !mv.captured;
        self.king &= !mv.captured;

        self.white_to_move = !self.white_to_move;

        if self.king & ALIVE_KING_ZONE == 0 || !self.has_moves() {
            self.set_win();
        } else if history.contains(&self.hash[0]) {
            self.set_draw();
        }
    }
}
