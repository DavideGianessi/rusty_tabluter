use std::io::{Read, Write};
use std::net::TcpStream;

use serde::{Deserialize, Serialize};

const WHITE_PORT: u16 = 5800;
const BLACK_PORT: u16 = 5801;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Turn {
    WHITE,
    BLACK,
    WHITEWIN,
    BLACKWIN,
    DRAW,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTablut {
    pub board: Vec<Vec<String>>,
    pub turn: Turn,
}

#[derive(Debug, Serialize)]
struct Action {
    from: String,
    to: String,
    turn: Turn,
}

#[derive(Debug, Clone)]
pub struct Move {
    pub from_col: usize,
    pub from_row: usize,
    pub to_col: usize,
    pub to_row: usize,
}

pub struct TablutClient {
    stream: TcpStream,
    pub role: Turn,
}

impl TablutClient {

    pub fn connect(server_ip: &str, role: Turn) -> std::io::Result<Self> {

        let port = match role {
            Turn::WHITE => WHITE_PORT,
            Turn::BLACK => BLACK_PORT,
            _ => panic!("Invalid role"),
        };

        let stream = TcpStream::connect(format!("{}:{}", server_ip, port))?;

        Ok(Self { stream, role })
    }

    pub fn declare_name(&mut self, name: &str) -> std::io::Result<()> {
        let json = serde_json::to_string(name).unwrap();
        write_string(&mut self.stream, &json)
    }

    pub fn read_state(&mut self) -> std::io::Result<StateTablut> {

        let msg = read_string(&mut self.stream)?;

        let state: StateTablut =
            serde_json::from_str(&msg)
                .expect("Invalid state JSON");

        Ok(state)
    }

    pub fn send_move(&mut self, mv: Move) -> std::io::Result<()> {

        let action = Action {
            from: coord_to_string(mv.from_col, mv.from_row),
            to: coord_to_string(mv.to_col, mv.to_row),
            turn: self.role.clone(),
        };

        let json = serde_json::to_string(&action).unwrap();

        write_string(&mut self.stream, &json)
    }
}

fn coord_to_string(col: usize, row: usize) -> String {

    let letter = (b'a' + col as u8) as char;
    let number = row + 1;

    format!("{}{}", letter, number)
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

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;

    Ok(String::from_utf8(buf).unwrap())
}
