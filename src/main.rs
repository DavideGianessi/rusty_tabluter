use primitive_types::U256;

mod board;
mod rot_table;
mod search;
mod eval;
mod debug;
mod stats;
mod client;
mod interactive;

use crate::board::{canonize, generate_moves, get_printable_board, parse_position,extract_move};
use crate::search::{search};
use crate::client::{...};

fn main() {

    let mut client =
        TablutClient::connect("127.0.0.1", Turn::WHITE)
            .expect("connection failed");

    client.declare_name("RustBot")
        .unwrap();

    loop {

        let state = client.read_state().unwrap();

        match state.turn {

            Turn::WHITE if client.role == Turn::WHITE => {

                let mv = decide_move(&state);

                client.send_move(mv).unwrap();
            }

            Turn::BLACK if client.role == Turn::BLACK => {

                let mv = decide_move(&state);

                client.send_move(mv).unwrap();
            }

            Turn::WHITEWIN => {
                println!("WHITE WIN");
                break;
            }

            Turn::BLACKWIN => {
                println!("BLACK WIN");
                break;
            }

            Turn::DRAW => {
                println!("DRAW");
                break;
            }

            _ => {}
        }
    }
}
