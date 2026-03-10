use primitive_types::U256;
use std::io::{self, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};

use crate::board::{generate_moves, get_printable_board, parse_position};
use crate::search::{search};

pub fn interactive() {

    let start_position = "OOOBBBOOO\n\
OOOOBOOOO\n\
OOOOWOOOO\n\
BOOOWOOOB\n\
BBWWKWWBB\n\
BOOOWOOOB\n\
OOOOWOOOO\n\
OOOOBOOOO\n\
OOOBBBOOO\n\
turn: W";

    let mut state = parse_position(start_position);

    let mut history: Vec<U256> = Vec::new();
    let mut moves: Vec<U256> = generate_moves(state, &history);
    let mut selected: usize = 0;

    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut keys = stdin.keys();

    loop {
        write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();

        write!(stdout, "STATE: {}\r\n", state).unwrap();

        let board_string = get_printable_board(state);
        write!(stdout, "{}", board_string).unwrap();

        write!(stdout, "\r\nMoves available: {}\r\n", moves.len()).unwrap();
        write!(stdout, "Selected: {}\r\n", selected).unwrap();
        //write!(stdout, "move chosen: {}\r\n", extract_move(state,moves[selected]).unwrap()).unwrap();

        write!(stdout, "\r\nPreview: {}\r\n", moves[selected]).unwrap();
        let selected_board_string = get_printable_board(moves[selected]);
        write!(stdout, "{}", selected_board_string).unwrap();

        write!(stdout, "\r\nControls:\r\n").unwrap();
        write!(stdout, "h/l = prev/next move\r\n").unwrap();
        write!(stdout, "j = commit move\r\n").unwrap();
        write!(stdout, "k = undo\r\n").unwrap();
        write!(stdout, "s = evaluate\r\n").unwrap();
        write!(stdout, "q = quit\r\n").unwrap();

        stdout.flush().unwrap();

        let key = match keys.next() {
            Some(Ok(k)) => k,
            _ => continue,
        };

        match key {
            termion::event::Key::Char('q') => break,

            termion::event::Key::Char('h') => {
                if selected > 0 {
                    selected -= 1;
                }
            }

            termion::event::Key::Char('l') => {
                if selected + 1 < moves.len() {
                    selected += 1;
                }
            }

            termion::event::Key::Char('j') => {
                if !moves.is_empty() {
                    history.push(state);
                    state = moves[selected];
                    moves = generate_moves(state,&history);
                    selected = 0;
                }
            }

            termion::event::Key::Char('k') => {
                if let Some(prev) = history.pop() {
                    state = prev;
                    moves = generate_moves(state,&history);
                    selected = 0;
                }
            }
            termion::event::Key::Char('s') => {
                if !moves.is_empty() {
                    let history_clone = history.clone();
                    let result = search(state, &history_clone);
                    if let Some(best_state) = result.best_move {
                        if let Some(index) = moves.iter().position(|&m| m == best_state) {
                            selected = index;
                        }
                        write!(
                            stdout,
                            "\r\nEngine value: {:.3}\r\n",
                            result.value
                        ).unwrap();
                        stdout.flush().unwrap();
                    }
                }
            }

            _ => {}
        }
    }
}
