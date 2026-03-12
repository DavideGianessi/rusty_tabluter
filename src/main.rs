//use std::env;
//use std::io::{self,BufRead};
//use std::str::FromStr;

//use primitive_types::U256;

mod board;
mod zobrist_keys;
//mod search;
//mod eval;
//mod debug;
//mod stats;
//mod client;
mod interactive;

//use crate::board::{format_position, parse_position, extract_move, get_white_win, get_black_win, get_draw, turn};
//use crate::search::{search,debug_search};
//use crate::client::{connect,get_state,send_move};
use crate::interactive::interactive;

/*
fn print_help() {
    println!("Usage:");

    println!("  bot --interactive");
    println!("      Start interactive debugging mode.");

    println!("  bot -s [U256]");
    println!("      Run a single search with debug output.");
    println!("      If U256 is provided, it is used as the state.");
    println!("      Otherwise a board is read from stdin using parse_position().");

    println!("  bot <white|black> <time_limit> <server_ip>");
    println!("      Connect to server and play normally.");

    println!("  bot -h");
    println!("      Show this help.");
}

fn parse_u256(arg: &str) -> Option<U256> {
    if let Ok(v) = U256::from_str(arg) {
        return Some(v);
    }
    if let Ok(v) = U256::from_str_radix(arg, 16) {
        return Some(v);
    }
    None
}

fn read_board() -> U256 {
    let stdin = io::stdin();
    let mut lines = Vec::new();

    for line in stdin.lock().lines().take(10) {
        lines.push(line.unwrap());
    }

    let board_string = lines.join("\n");
    parse_position(&board_string)
}
*/

fn main() {
    interactive();
    /*
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty(){
        print_help();
        return;
    }

    match args[0].as_str() {
        "-h" | "--help" => {
            print_help();
        }

        "--interactive" => {
            interactive();
        }

        "-s" => {
            if args.len() > 2 {
                print_help();
                return;
            }

            let state = if args.len() == 2 {
                match parse_u256(&args[1]) {
                    Some(v) => v,
                    None => {
                        println!("Invalid U256");
                        return;
                    }
                }
            } else {
                read_board()
            };

            let history: Vec<U256> = Vec::new();
            debug_search(state, &history);
        }

        _ => {
            if args.len() != 3 {
                print_help();
                return;
            }

            let color = args[0].to_lowercase();

            let is_white = match color.as_str() {
                "white" => true,
                "black" => false,
                _ => {
                    print_help();
                    return;
                }
            };


            let _time_limit: u64 = match args[1].parse() {
                Ok(v) => v,
                Err(_) => {
                    print_help();
                    return;
                }
            };

            let server_ip = &args[2];

            let res = connect(server_ip, is_white);
            if !res.is_ok() {
                println!("network error");
            }

            let mut history: Vec<U256> = Vec::new();

            loop {
                let state: U256 = get_state().unwrap();
                println!("state received: {}\n{}", state,format_position(state));

                if state == get_white_win() {
                    println!("WHITE WIN");
                    return;
                }
                if state == get_black_win() {
                    println!("BLACK WIN");
                    return;
                }
                if state == get_draw() {
                    println!("DRAW");
                    return;
                }

                if turn(state) != is_white {
                    history.push(state);
                    continue;
                }

                let result = search(state, &history);

                let best_state = result.best_move.unwrap();
                let best_value = result.value;

                let (start_row, start_col, end_row, end_col) = extract_move(state, best_state, &history).unwrap();
                history.push(state);

                println!(
                    "mossa: {} {} -> {} {}\nvalue: {}",
                    start_row, start_col, end_row, end_col,best_value
                );

                let res = send_move(start_row, start_col, end_row, end_col);

                if !res.is_ok() {
                    println!("network error");
                    return;
                }

            }
        }
    }
*/
}

