use mio::{Token, net::TcpStream};

use crate::{ProtocolState, buffer::Buf, data::slot::Slot};

pub struct Bot {
    pub token: Token,
    pub stream: TcpStream,
    pub name: String,
    pub id: u32,
    pub entity_id: u32,
    pub compression_threshold: i32,
    pub state: ProtocolState,
    pub kicked: bool,
    pub teleported: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub buffering_buf: Buf,
    pub joined: bool,
    pub state_id: i32,
    pub inventory: Vec<Slot>,
}
