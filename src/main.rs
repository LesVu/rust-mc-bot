mod nets;
mod packet_processors;
mod packet_utils;
mod states;

use crate::nets::stream::Stream;
use crate::packet_utils::Buf;
use crate::states::login;
use libdeflater::{CompressionLvl, Compressor, Decompressor};
use mio::net::TcpStream;
use mio::{Events, Interest, Poll, Token};
use states::play;

use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use std::{env, net::ToSocketAddrs};
use uuid::Uuid;

const SHOULD_MOVE: bool = true;

const PROTOCOL_VERSION: u32 = 776;
const BOT_NAME: &str = "BooRE1";

type Error = Box<dyn std::error::Error + Send + Sync>;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        let name = args.get(0).unwrap();
        println!("usage: {} <ip:port> <count> [threads]", name);
        println!("example: {} localhost:25565 500", name);
        return Ok(());
    }

    let arg1 = args.get(1).unwrap();

    let mut addrs = None;

    if addrs.is_none() {
        let mut parts = arg1.split(':');
        let ip = parts.next().expect("no ip provided");
        let port = parts
            .next()
            .map(|port_string| port_string.parse().expect("invalid port"))
            .unwrap_or(25565u16);

        let server = (ip, port)
            .to_socket_addrs()
            .expect("Not a socket address")
            .next()
            .expect("No socket address found");

        addrs = Some(Address::new(server));
    }

    // Cant be none because it would have panicked earlier
    let addrs = addrs.unwrap();

    start_bot(addrs, BOT_NAME.to_string());

    Ok(())
}

pub struct Compression {
    compressor: Compressor,
    decompressor: Decompressor,
}

pub struct Bot {
    pub token: Token,
    pub stream: Stream,
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
}

#[derive(Debug, Clone, Copy)]
pub enum ProtocolState {
    Status,
    Login,
    Config,
    Play,
}

pub fn start_bot(addrs: Address, name: String) {
    let mut poll = Poll::new().expect("could not unwrap poll");
    let mut events = Events::with_capacity(16);

    println!("{:?}", addrs);

    let token = Token(0);
    let mut bot = Bot {
        token,
        stream: addrs.connect(),
        name,
        id: 0,
        entity_id: 0,
        compression_threshold: 0,
        state: ProtocolState::Login,
        kicked: false,
        teleported: false,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        buffering_buf: Buf::with_length(200),
        joined: false,
    };

    poll.registry()
        .register(
            &mut bot.stream,
            token,
            Interest::READABLE | Interest::WRITABLE,
        )
        .expect("could not register stream");

    let mut packet_buf = Buf::with_length(2000);
    let mut uncompressed_buf = Buf::with_length(2000);
    let mut compression = Compression {
        compressor: Compressor::new(CompressionLvl::default()),
        decompressor: Decompressor::new(),
    };

    let dur = Duration::from_millis(50);

    while !bot.kicked {
        let ins = Instant::now();

        poll.poll(&mut events, Some(dur)).expect("couldn't poll");

        for event in events.iter() {
            if event.is_writable() && !bot.joined {
                bot.joined = true;

                // socket ops
                bot.stream.set_ops();

                //login sequence
                let buf = login::write_handshake_packet(PROTOCOL_VERSION, "".to_string(), 0, 2);
                bot.send_packet(buf, &mut compression);

                let uuid: u128 = Uuid::new_v4().as_u128();
                let buf = login::write_login_start_packet(&bot.name, uuid);
                bot.send_packet(buf, &mut compression);

                println!("bot \"{}\" joined", bot.name);
            }

            if event.is_readable() && bot.joined {
                nets::net::process_packet(
                    &mut bot,
                    &mut packet_buf,
                    &mut uncompressed_buf,
                    &mut compression,
                );
            }
        }

        if bot.kicked {
            println!("{} disconnected", bot.name);
            break;
        }

        // Ticking actions
        if SHOULD_MOVE && bot.teleported {
            // bot.x += rand::random::<f64>() * 1.0 - 0.5;
            // bot.z += rand::random::<f64>() * 1.0 - 0.5;
            // bot.send_packet(play::write_current_pos(&bot), &mut compression);

            // Sneak (0x20 = Shift / Sneak flag)
            bot.send_packet(play::write_player_input(0x20), &mut compression);
            println!("Sneaking");

            // Hold for 1 second
            std::thread::sleep(Duration::from_secs(2));

            // Unsneak (clear sneak flag)
            bot.send_packet(play::write_player_input(0x00), &mut compression);
            println!("Un Sneak");
            // Hold for 1 second
            std::thread::sleep(Duration::from_secs(2));
        }

        let elapsed = ins.elapsed();
        if elapsed < dur {
            std::thread::sleep(dur - elapsed);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Address(SocketAddr);

impl Address {
    pub fn new(addr: SocketAddr) -> Self {
        Self(addr)
    }
    pub fn connect(&self) -> Stream {
        Stream::new(TcpStream::connect(self.0.to_owned()).expect("Could not connect to the server"))
    }
}
