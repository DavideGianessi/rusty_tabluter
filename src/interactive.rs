use std::io::{self, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor, style};
use std::time::Duration;

use crate::board::State;
use crate::eval::evaluate;
use crate::search::search;
use crate::weights::Weights;

const CITADELS: u128 = 264600106062302366670904;
const THRONE: u128 = 1099511627776;
const ESCAPE: u128 = 937403599922078434067142;

fn is_citadel(row: u8, col: u8) -> bool {
    ((CITADELS >> (row * 9 + col)) & 1) != 0
}
fn is_throne(row: u8, col: u8) -> bool {
    ((THRONE >> (row * 9 + col)) & 1) != 0
}
fn is_escape(row: u8, col: u8) -> bool {
    ((ESCAPE >> (row * 9 + col)) & 1) != 0
}

pub fn interactive() {
    let mut state = State::new();
    let mut history: Vec<State> = Vec::new();
    let mut history_real: Vec<u64> = Vec::new();
    let weights = Weights::new();

    let mut cursor_r: i8 = 4;
    let mut cursor_c: i8 = 4;
    let mut selected_piece: Option<(u8, u8)> = None;
    let mut search_val: Option<i32> = None;

    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut keys = stdin.keys();

    loop {
        write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
        if state.win || state.draw {
            let mut msg = "";
            if state.win && !state.white_to_move {
                msg = "VITTORIA BIANCO";
            }
            if state.win && state.white_to_move {
                msg = "VITTORIA NERO";
            }
            if state.draw {
                msg = "PATTA";
            }
            write!(stdout, "\r\n\n   {}\r\n", msg.to_string()).unwrap();
            write!(stdout, "\r\n   [U] Undo | [R] Reset | [Q] Quit\r\n").unwrap();
            stdout.flush().unwrap();
        } else {
            let mut all_moves = Vec::new();
            state.generate_moves(&mut all_moves);

            let mut capture_mask: u128 = 0;
            let legal_targets: Vec<(u8, u8)> = if let Some((sr, sc)) = selected_piece {
                let targets: Vec<(u8, u8)> = all_moves
                    .iter()
                    .filter(|m| m.fr == sr && m.fc == sc)
                    .map(|m| (m.tr, m.tc))
                    .collect();

                if let Some(mv) = all_moves.iter().find(|m| {
                    m.fr == sr && m.fc == sc && m.tr == cursor_r as u8 && m.tc == cursor_c as u8
                }) {
                    capture_mask = mv.captured;
                }
                targets
            } else {
                Vec::new()
            };

            render_board(
                &mut stdout,
                &state,
                cursor_r as u8,
                cursor_c as u8,
                selected_piece,
                &legal_targets,
                capture_mask,
            );

            let (val, instability) = evaluate(&state, &weights);

            write!(
                stdout,
                "\r\n Turno: {} | Hash: {} | Eval: {} | Instability: {}\r\n",
                if state.white_to_move {
                    "BIANCO"
                } else {
                    "NERO"
                },
                state.hash(),
                val,
                instability
            )
            .unwrap();
            if let Some(s_val) = search_val {
                write!(stdout, "Suggestion Score: {}\r\n", s_val).unwrap();
            } else {
                write!(stdout, "\r\n").unwrap();
            }
            write!(
                stdout,
                " [WASD/HJKL] Muovi | [SPAZIO] Seleziona | [G] Search | [U] Undo | [R] Reset | [Q] Quit\r\n"
            )
            .unwrap();
            stdout.flush().unwrap();
        }

        let key = match keys.next() {
            Some(Ok(k)) => k,
            _ => continue,
        };

        match key {
            Key::Char('q') => break,
            Key::Char('r') => {
                state = State::new();
                history.clear();
                history_real.clear();
                selected_piece = None;
                cursor_r = 4;
                cursor_c = 4;
                search_val = None;
            }
            Key::Char('g') => {
                let result = search(&state, &history_real, &weights,Duration::from_secs(2), false);
                if let Some(mv) = result.best_move {
                    selected_piece = Some((mv.fr, mv.fc));
                    cursor_r = mv.tr as i8;
                    cursor_c = mv.tc as i8;
                    search_val = Some(result.value);
                }
            }
            Key::Char('u') => {
                if let Some(prev) = history.pop() {
                    state = prev;
                    selected_piece = None;
                    history_real.pop();
                    search_val = None;
                }
            }

            Key::Char('w') | Key::Char('k') => {
                if cursor_r > 0 {
                    cursor_r -= 1
                }
            }
            Key::Char('s') | Key::Char('j') => {
                if cursor_r < 8 {
                    cursor_r += 1
                }
            }
            Key::Char('a') | Key::Char('h') => {
                if cursor_c > 0 {
                    cursor_c -= 1
                }
            }
            Key::Char('d') | Key::Char('l') => {
                if cursor_c < 8 {
                    cursor_c += 1
                }
            }

            Key::Char(' ') => {
                let r = cursor_r as u8;
                let c = cursor_c as u8;
                if let Some((sr, sc)) = selected_piece {
                    let mut moves = Vec::new();
                    state.generate_moves(&mut moves);
                    if let Some(mv) = moves
                        .iter()
                        .find(|m| m.fr == sr && m.fc == sc && m.tr == r && m.tc == c)
                    {
                        history.push(state);
                        let the_hash = state.hash();
                        state.apply_move(mv, &history_real);
                        history_real.push(the_hash);
                        selected_piece = None;
                        search_val = None;
                    } else {
                        selected_piece = None;
                        search_val = None
                    }
                } else {
                    if (state.white_to_move && state.is_white(r, c))
                        || (!state.white_to_move && state.is_black(r, c))
                    {
                        selected_piece = Some((r, c));
                    }
                }
            }
            _ => {}
        }
    }
}

fn render_board<W: Write>(
    stdout: &mut W,
    state: &State,
    cur_r: u8,
    cur_c: u8,
    sel: Option<(u8, u8)>,
    targets: &[(u8, u8)],
    capture_mask: u128,
) {
    let header = "   +-----+-----+-----+-----+-----+-----+-----+-----+-----+\r\n";
    write!(stdout, "{}", header).unwrap();

    for r in 0..9 {
        write!(stdout, " {} |", r).unwrap();
        for c in 0..9 {
            let idx = r * 9 + c;
            let bit = 1u128 << idx;

            let is_cursor = r == cur_r && c == cur_c;
            let is_target = targets.contains(&(r, c));
            let is_captured = (capture_mask & bit) != 0;

            let bg = if is_captured {
                "\x1b[48;5;124m"
            } else if is_cursor {
                "\x1b[48;5;248m"
            } else if is_target {
                "\x1b[48;5;22m"
            } else if is_throne(r, c) {
                "\x1b[48;5;178m"
            } else if is_citadel(r, c) {
                "\x1b[48;5;243m"
            } else if is_escape(r, c) {
                "\x1b[48;5;39m"
            } else {
                ""
            };

            let reset_bg = if bg != "" { "\x1b[49m" } else { "" };

            let piece = if state.is_king(r, c) {
                format!(
                    "{}{}{} K {}",
                    color::Fg(color::Yellow),
                    style::Bold,
                    bg,
                    color::Fg(color::Reset)
                )
            } else if state.is_white(r, c) {
                format!(
                    "{}{}{} W {}",
                    color::Fg(color::White),
                    style::Bold,
                    bg,
                    color::Fg(color::Reset)
                )
            } else if state.is_black(r, c) {
                format!(
                    "{}{}{} B {}",
                    color::Fg(color::LightRed),
                    style::Bold,
                    bg,
                    color::Fg(color::Reset)
                )
            } else {
                format!("{}   ", bg)
            };

            let l_brk = if sel == Some((r, c)) {
                ">"
            } else if is_cursor {
                "["
            } else {
                " "
            };
            let r_brk = if sel == Some((r, c)) {
                "<"
            } else if is_cursor {
                "]"
            } else {
                " "
            };

            write!(stdout, "{}{}{}{}{}|", bg, l_brk, piece, r_brk, reset_bg).unwrap();
        }
        write!(stdout, "\r\n{}", header).unwrap();
    }
}
