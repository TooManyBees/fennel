use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::{Instant, Duration};
use std::thread;
use crossbeam_channel::{bounded, Sender, Receiver};
use log;
use femme::{self, LevelFilter};
// use telnet::Telnet;

struct Connection {
    stream: TcpStream,
    // stream: Telnet,
    // connected: bool,
    character: Option<Character>,
}

impl Connection {
    pub fn write(&mut self, message: &str) -> std::io::Result<()> {
        self.stream.write_all(message.as_bytes())
    }

    pub fn read(&mut self) -> std::io::Result<String> {
        unimplemented!()
        // self.stream.read_to_end()
    }
}

enum Nanny {
    None,
    GetName,
    GetPassword(String),
    NewCharacter(String),
    NewCharacterConfirmPassword(String, String)
}

struct Character {
    name: String,
    password_hash: String,
    timer: u16,
}

static PULSE_PER_SECOND: u32 = 3;
static PULSE_RATE_NS: u32 = 1_000_000_000 / PULSE_PER_SECOND;

fn game_loop(connection_receiver: Receiver<Connection>) -> std::io::Result<()> {
    let mut last_time = Instant::now();
    let mut connections = vec![];
    loop {
        last_time = Instant::now();

        // let in the new connections
        while let Ok(c) = connection_receiver.try_recv() {
            connections.push(c);
        }

        // handle input

        // update world

        // handle output
        for conn in &mut connections {
            let _ = conn.stream.write("pulse\r\n".as_bytes());
        }

        let now = Instant::now();
        let sleep_for = last_time + Duration::new(0, PULSE_RATE_NS) - now;
        thread::sleep(sleep_for);
    }
}

fn login() {

}

fn listen(listener: TcpListener, sender: Sender<Connection>) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                log::info!("New connection {:?}", &stream.peer_addr());
                let conn = Connection { stream };
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
    femme::with_level(LevelFilter::Info);

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
