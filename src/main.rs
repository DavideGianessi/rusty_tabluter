use primitive_types::U256;

mod board;
mod rot_table;
mod search;
mod eval;
mod debug;
mod stats;
mod client;
mod interactive;

use crate::board::{canonize, generate_moves, format_position, extract_move, get_white_win, get_black_win, get_draw};
use crate::search::{search};
use crate::client::{connect,get_state,send_move};

fn main() {
    let res = connect("localhost", true);
    if !res.is_ok() {
            println!("network error");
            return;
    }
    let mut history: Vec<U256> = Vec::new();

    loop {

        let state: U256 = get_state().unwrap();

        println!("state received: \n{}",format_position(state));
        if state == get_white_win() {
            println!("WHITE WIN");
            break;
        }

        if state == get_black_win() {
            println!("BLACK WIN");
            break;
        }

        if state == get_draw() {
            println!("DRAW");
            break;
        }

        let result = search(state, &history);
        println!("value: {} new_state_canonized: \n{}",result.value, format_position(result.best_move.unwrap()));
        let best_state = result.best_move.unwrap();
        history.push(canonize(state));
        history.push(canonize(best_state));
        let mv = extract_move(state,best_state).unwrap();
        let (start_row,start_col,end_row,end_col) = mv;
        println!("mossa: {} {} -> {} {}",start_row,start_col, end_row, end_col);
        let res = send_move(start_row,start_col,end_row,end_col);
        if !res.is_ok() {
                println!("network error");
                return;
        }
        let state = get_state().unwrap(); //mi restituisce lo stato dopo la mia mossa
        if state == get_white_win() {
            println!("WHITE WIN");
            break;
        }

        if state == get_black_win() {
            println!("BLACK WIN");
            break;
        }

        if state == get_draw() {
            println!("DRAW");
            break;
        }
    }
}
