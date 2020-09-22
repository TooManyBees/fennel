use crossbeam_channel::{bounded, Receiver};
use femme::{self, LevelFilter};
use generational_arena::Arena;
use log;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use fennel::{listen, Area, Character, Connection, Room, RoomId};

static PULSE_PER_SECOND: u32 = 3;
static PULSE_RATE_NS: u32 = 1_000_000_000 / PULSE_PER_SECOND;

fn load_areas(rooms: &mut HashMap<RoomId, Room>, areas: &mut Vec<Area>) {
    log::info!("Loading areas");
    // FIXME: daaaaaaang this is ugly
    // TODO: we're gonna iterate over every area name in some text file, rather than load a single area
    match Area::load("default") {
        Ok(mut area_def) => {
            let mut room_defs = Vec::new();
            std::mem::swap(&mut room_defs, &mut area_def.rooms);

            let area = Area::from_prototype(area_def);
            let area_idx = areas.len();
            areas.push(area);
            let area = &mut areas[area_idx];

            for room_def in room_defs {
                let room = Room::from_prototype(room_def, area_idx);
                let room_idx = room.id;
                rooms.insert(room.id, room);
                area.rooms.push(room_idx);
            }
        }
        Err(e) => log::error!("Error loading area {:?}", e),
    }
    log::info!("Loading areas: success");
}

fn game_loop(connection_receiver: Receiver<(Connection, Character)>) -> std::io::Result<()> {
    let mut last_time: Instant;
    let mut connections: Arena<Connection> = Arena::new();
    let mut characters: Arena<Character> = Arena::new();
    let mut areas = Vec::new();
    let mut rooms = HashMap::new();
    let mut mark_for_disconnect = Vec::new();

    load_areas(&mut rooms, &mut areas);

    println!("{:?}\n\n{:?}", areas, rooms);

    loop {
        last_time = Instant::now();

        // let in the new connections
        while let Ok((mut conn, char)) = connection_receiver.try_recv() {
            // FIXME: there must be a better test than a name comparison. Should I use a UUID or something?
            if let Some((char_index, _)) = characters.iter().find(|(_, c)| c.name() == char.name())
            {
                log::info!(
                    "Connection regained from {} for {}",
                    conn.addr(),
                    char.name()
                );
                // Take control of the char
                conn.character = Some(char_index);
                if let Some((conn_idx, conn)) = connections
                    .iter_mut()
                    .find(|(_, conn)| conn.character == Some(char_index))
                {
                    log::info!(
                        "Connection taken over by {} for {}",
                        conn.addr(),
                        char.name()
                    );
                    let _ = conn.write("Disconnected");
                    connections.remove(conn_idx);
                }
            } else {
                log::info!("New connection from {} for {}", conn.addr(), char.name());
                let char_idx = characters.insert(char);
                conn.character = Some(char_idx);
            }
            let _conn_idx = connections.insert(conn);
        }

        // handle input
        for (idx, conn) in &mut connections {
            if let Err(e) = conn.read() {
                match e.kind() {
                    ErrorKind::ConnectionAborted => {
                        let char_name = conn
                            .character
                            .and_then(|idx| characters.get(idx))
                            .map(|ch| ch.name());
                        log::info!(
                            "Connection went linkdead: {:?} from {}",
                            char_name,
                            conn.addr()
                        );
                        mark_for_disconnect.push(idx);
                    }
                    ErrorKind::WouldBlock => {} // explicitly okay no matter what happens to the catch-all branch
                    _ => log::warn!("Unexpected input read error from {}: {}", conn.addr(), e),
                }
            }
            // decrement lag

            // interpret input
        }

        for idx in &mark_for_disconnect {
            connections.remove(*idx);
        }
        mark_for_disconnect.clear();

        // update world

        // handle output
        for (_idx, conn) in &mut connections {
            if let Some(character) = conn.character.and_then(|idx| characters.get(idx)) {
                let pulse_output = format!(
                    "heartbeat for {} majesty {}",
                    character.pronoun().possessive(),
                    character.name()
                );
                let _ = conn.write(&pulse_output);
            }
        }

        let now = Instant::now();
        let next_pulse = last_time + Duration::new(0, PULSE_RATE_NS);
        let sleep_for = if now > next_pulse {
            next_pulse - now
        } else {
            Duration::new(0, 0)
        };
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
    thread::Builder::new()
        .name("listen & login".to_string())
        .spawn(move || {
            listen(listener, login_queue_sender);
        })?;
    game_loop(login_queue_receiver)?;

    Ok(())
}
