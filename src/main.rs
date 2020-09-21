mod character;
mod pronoun;
mod listener;

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::time::{Instant, Duration};
use std::thread;
use crossbeam_channel::{bounded, Receiver};
use log;
use femme::{self, LevelFilter};
use generational_arena::{Arena, Index};

use character::Character;

pub struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    character: Option<Index>,
    in_buffer: [u8; 256],
    input: Option<String>, // TODO: do this better
    output: Option<String>,
}

impl Connection {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Connection {
        Connection {
            stream,
            addr,
            character: None,
            in_buffer: [0; 256],
            input: None,
            output: None,
        }
    }

    pub fn write(&mut self, message: &str) -> std::io::Result<()> {
        self.stream.write_all(message.as_bytes())
    }

    pub fn read(&mut self) -> std::io::Result<()> {
        // FIXME: prevent input overflows; max length should be 256
        let n = self.stream.read(&mut self.in_buffer)?;
        let s = String::from_utf8(self.in_buffer[..n].to_vec())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
        self.input = Some(s);
        Ok(())
    }
}

static PULSE_PER_SECOND: u32 = 3;
static PULSE_RATE_NS: u32 = 1_000_000_000 / PULSE_PER_SECOND;

fn game_loop(connection_receiver: Receiver<(Connection, Character)>) -> std::io::Result<()> {
    let mut last_time = Instant::now();
    let mut connections = Arena::new();
    let mut characters = Arena::new();
    let mut mark_for_disconnect = Vec::new();
    loop {
        last_time = Instant::now();

        // let in the new connections
        while let Ok((mut conn, char)) = connection_receiver.try_recv() {
            log::info!("New connection from {} for {}", conn.addr, char.name());
            // TODO: what if the character is already ingame? Take control of linkdead char,
            // or boot whoever is currently playing the char and take control from them
            let char_idx = characters.insert(char);
            conn.character = Some(char_idx);
            let _conn_idx = connections.insert(conn);
        }

        // handle input
        for (idx, conn) in &mut connections {
            if let Err(e) = conn.read() {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    // TODO: this doesn't need to be a warning
                    log::warn!("Error reading input from user {:?}", e.kind());
                    // TODO: marking someone for disconnect because their input was bad is maybe too much
                    mark_for_disconnect.push(idx);
                }
            }
            // decrement lag

            // interpret input
        }

        for idx in &mark_for_disconnect {
            connections.remove(*idx);
        }

        // update world

        // handle output
        for (_idx, conn) in &mut connections {
            if let Some(character) = conn.character.and_then(|idx| characters.get(idx)) {
                let pulse_output = format!("heartbeat for {} majesty {}", character.pronoun().possessive(), character.name());
                let _ = conn.stream.write(pulse_output.as_bytes());
            }
        }

        let now = Instant::now();
        let sleep_for = last_time + Duration::new(0, PULSE_RATE_NS) - now;
        thread::sleep(sleep_for);
    }
}

fn main() -> std::io::Result<()> {
    femme::with_level(LevelFilter::Debug);

    // load everything

    let port = 3001;
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let listener = smol::Async::new(listener)?;
    log::info!("Listening on port {}", port);

    let (login_queue_sender, login_queue_receiver) = bounded(20);
    thread::Builder::new().name("listen & login".to_string()).spawn(move || {
        listener::listen(listener, login_queue_sender);
    })?;
    game_loop(login_queue_receiver)?;

    Ok(())
}
