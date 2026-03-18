use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Mutex, OnceLock};

use serde_json::Value;

use crate::board::State;

static STREAM: OnceLock<Mutex<TcpStream>> = OnceLock::new();
static ROLE_IS_WHITE: OnceLock<bool> = OnceLock::new();

const WHITE_PORT: u16 = 5800;
const BLACK_PORT: u16 = 5801;

pub fn connect(server_ip: &str, role_white: bool) -> std::io::Result<()> {
    let port = if role_white { WHITE_PORT } else { BLACK_PORT };

    let stream = TcpStream::connect(format!("{}:{}", server_ip, port))?;

    STREAM
        .set(Mutex::new(stream))
        .expect("STREAM already initialized");

    ROLE_IS_WHITE
        .set(role_white)
        .expect("ROLE already initialized");

    send_name("RustBot")?;

    Ok(())
}

fn send_name(name: &str) -> std::io::Result<()> {
    let mut stream = STREAM.get().unwrap().lock().unwrap();
    write_string(&mut stream, name)
}
pub fn get_game_state() -> std::io::Result<State> {
    let mut stream = STREAM.get().unwrap().lock().unwrap();
    
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;

    let v: Value = serde_json::from_str(&String::from_utf8_lossy(&buffer))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let turn_str = v["turn"].as_str().unwrap_or("");
    
    let mut white = 0u128;
    let mut black = 0u128;
    let mut king = 0u128;
    let white_to_move = turn_str == "WHITE";

    if turn_str == "WHITE" || turn_str == "BLACK" {
        if let Some(board_array) = v["board"].as_array() {
            for r in 0..9 {
                if let Some(row_array) = board_array[r].as_array() {
                    for c in 0..9 {
                        let bit = 1u128 << (r * 9 + c);
                        match row_array[c].as_str().unwrap_or("") {
                            "WHITE" => white |= bit,
                            "BLACK" => black |= bit,
                            "KING"  => {
                                white |= bit;
                                king = bit;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    let mut state = State {
        white,
        black,
        king,
        white_to_move,
        win: false,
        draw: false,
        hash: 0,
    };

    match turn_str {
        "WHITEWIN"  => {state.win = true; state.white_to_move = false},
        "BLACKWIN" => {state.win = true; state.white_to_move = true},
        "DRAW" => state.draw = true,
        _ => state.compute_full_hash(),
    }

    Ok(state)
}

pub fn send_move(
    row_start: usize,
    col_start: usize,
    row_end: usize,
    col_end: usize,
) -> std::io::Result<()> {
    let mut stream = STREAM.get().unwrap().lock().unwrap();

    let from = format!(
        "{}{}",
        (b'a' + col_start as u8) as char,
        row_start + 1
    );

    let to = format!(
        "{}{}",
        (b'a' + col_end as u8) as char,
        row_end + 1
    );

    let role = if *ROLE_IS_WHITE.get().unwrap() {
        "WHITE"
    } else {
        "BLACK"
    };

    let action = serde_json::json!({
        "from": from,
        "to": to,
        "turn": role
    });

    let json = serde_json::to_string(&action).unwrap();

    write_string(&mut stream, &json)
}

fn write_string(stream: &mut TcpStream, s: &str) -> std::io::Result<()> {
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;

    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(bytes)?;
    Ok(())
}
