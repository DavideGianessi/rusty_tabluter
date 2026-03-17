use std::env;

mod board;
mod zobrist_keys;
mod search;
mod eval;
mod weights;
//mod debug;
mod stats;
mod client;
mod interactive;

use crate::board::State;
use crate::search::{search};
use crate::client::{connect,get_game_state,send_move};
use crate::interactive::interactive;
use crate::weights::Weights;
use crate::stats::{reset_stats,print_stats_string};

fn print_help() {
    println!("Usage:");

    println!("  bot --interactive");
    println!("      Start interactive debugging mode.");

    println!("  bot <white|black> <time_limit> <server_ip>");
    println!("      Connect to server and play normally.");

    println!("  bot -h");
    println!("      Show this help.");
}


fn main() {
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

            let mut history: Vec<u64> = Vec::new();
            let weights = Weights::new();

            loop {
                let state: State = get_game_state().unwrap();
                println!("state received:\n{}", state.to_position_string());

                if state.win && !state.white_to_move{
                    println!("WHITE WIN");
                    return;
                }
                if state.win && state.white_to_move{
                    println!("BLACK WIN");
                    return;
                }
                if state.draw {
                    println!("DRAW");
                    return;
                }

                if state.white_to_move != is_white {
                    history.push(state.hash());
                    continue;
                }

                reset_stats();
                let result = search(state, &history, &weights);

                let best_move = result.best_move.unwrap();
                let best_value = result.value;

                let start_row = best_move.fr;
                let start_col = best_move.fc;
                let end_row = best_move.tr;
                let end_col = best_move.tc;
                history.push(state.hash());

                println!(
                    "mossa: {} {} -> {} {}\nvalue: {}",
                    start_row, start_col, end_row, end_col,best_value
                );
                println!("{}",print_stats_string());

                let res = send_move(start_row as usize, start_col as usize, end_row as usize, end_col as usize);

                if !res.is_ok() {
                    println!("network error");
                    return;
                }

            }
        }
    }
}

