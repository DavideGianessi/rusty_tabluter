use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Mutex, OnceLock};

use primitive_types::U256;
use serde_json::Value;

use crate::board::{get_white_win, get_black_win, get_draw};

static STREAM: OnceLock<Mutex<TcpStream>> = OnceLock::new();
static ROLE_IS_WHITE: OnceLock<bool> = OnceLock::new();

const WHITE_PORT: u16 = 5800;
const BLACK_PORT: u16 = 5801;

fn index(row: usize, col: usize) -> usize {
    row * 9 + col
}

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

/*
pub fn get_state() -> std::io::Result<U256> {
    let mut stream = STREAM.get().unwrap().lock().unwrap();
    let json = read_string(&mut stream)?;
    Ok(parse_json_to_bitboard(&json))
}*/
pub fn get_state() -> std::io::Result<U256> {
    let mut stream = STREAM.get().unwrap().lock().unwrap();

    let json = read_string(&mut stream)?;

    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let turn = v["turn"].as_str().unwrap();

    let state = match turn {
        "WHITE" | "BLACK" => parse_json_to_bitboard(&json),

        "WHITEWIN" => get_white_win(),

        "BLACKWIN" => get_black_win(),

        "DRAW" => get_draw(),

        _ => panic!("Unknown turn received from server: {}", turn),
    };

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

fn read_string(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;

    let len = u32::from_be_bytes(len_buf) as usize;

    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;

    Ok(String::from_utf8(buffer).unwrap())
}

fn parse_json_to_bitboard(input: &str) -> U256 {
    let v: Value = serde_json::from_str(input).unwrap();

    let board = v["board"].as_array().unwrap();
    let turn = v["turn"].as_str().unwrap();

    let mut state = U256::zero();
    let mut k_row = 0usize;
    let mut k_col = 0usize;

    for row in 0..9 {
        let row_arr = board[row].as_array().unwrap();

        for col in 0..9 {
            let cell = row_arr[col].as_str().unwrap();
            let idx = index(row, col);

            match cell {
                "WHITE" => state |= U256::one() << idx,
                "BLACK" => state |= U256::one() << (81 + idx),
                "KING" => {
                    state |= U256::one() << idx;
                    k_row = row;
                    k_col = col;
                }
                _ => {}
            }
        }
    }

    state |= U256::from(k_row as u32) << (2 * 81);
    state |= U256::from(k_col as u32) << (2 * 81 + 4);

    if turn == "WHITE" {
        state |= U256::one() << (2 * 81 + 8);
    }

    state
}
