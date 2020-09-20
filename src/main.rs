mod character;
mod pronoun;
mod login;

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::{Instant, Duration};
use std::thread;
use crossbeam_channel::{bounded, Sender, Receiver};
use log;
use femme::{self, LevelFilter};

use character::Character;
use pronoun::Pronoun;
use login::{ConnectState, login};

pub struct Connection {
    stream: TcpStream,
    character: Option<Character>,
    in_buffer: [u8; 256],
    input: Option<String>, // TODO: do this better
    output: Option<String>,
    connect_state: ConnectState,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            character: None,
            in_buffer: [0; 256],
            input: None,
            output: None,
            connect_state: ConnectState::GetName,
        }
    }

    pub fn write(&mut self, message: &str) -> std::io::Result<()> {
        self.stream.write_all(message.as_bytes())
    }

    pub fn read(&mut self) -> std::io::Result<()> {
        // FIXME: prevent input overflows; max length should be 256
        let n = self.stream.read(&mut self.in_buffer)?;
        let s = String::from_utf8(self.in_buffer[..n].to_vec()).unwrap();
        self.input = Some(s);
        Ok(())
    }
}

static PULSE_PER_SECOND: u32 = 3;
static PULSE_RATE_NS: u32 = 1_000_000_000 / PULSE_PER_SECOND;

fn game_loop(connection_receiver: Receiver<Connection>) -> std::io::Result<()> {
    let mut last_time = Instant::now();
    let mut connections = vec![];
    loop {
        last_time = Instant::now();

        // let in the new connections
        while let Ok(mut c) = connection_receiver.try_recv() {
            c.write("What is your name? ");
            connections.push(c);
        }

        // handle input
        for conn in &mut connections {
            if let Err(e) = conn.read() {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    // TODO: this doesn't need to be a warning
                    log::warn!("Error reading input from user {}", e);
                    // TODO: disconnect ppl?
                }
            }
            if conn.character.is_none() {
                if let Err(e) = login(conn) {
                    // close char and disconnect
                }
                continue;
            }
            // decrement lag

            // interpret input
        }

        // update world

        // handle output
        for conn in &mut connections {
            if let Some(character) = &conn.character {
                let pulse_output = format!("pulse for {} majesty {}", character.pronoun().possessive(), character.name());
                let _ = conn.stream.write(pulse_output.as_bytes());
            }
        }

        let now = Instant::now();
        let sleep_for = last_time + Duration::new(0, PULSE_RATE_NS) - now;
        thread::sleep(sleep_for);
    }
}

fn listen(listener: TcpListener, sender: Sender<Connection>) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                log::info!("New connection {:?}", &stream.peer_addr());
                let _ = stream.set_nonblocking(true);
                let conn = Connection::new(stream);
                if let Err(e) = sender.send(conn) {
                    log::error!("Couldn't transfer a new connection to main thread");
                    let _ = e.into_inner().stream.write("Unexpected error connecting\r\n".as_bytes());
                }
            },
            Err(e) => log::error!("{}", e),
        }
    }
}

fn main() -> std::io::Result<()> {
    femme::with_level(LevelFilter::Debug);

    // load everything

    let port = 3001;
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    log::info!("Listening on port {}", port);

    let (login_queue_sender, login_queue_receiver) = bounded(20);
    thread::Builder::new().name("listen & login".to_string()).spawn(move || {
        listen(listener, login_queue_sender);
    });
    game_loop(login_queue_receiver)?;

    Ok(())
}
