use crossbeam_channel::{bounded, Receiver};
use femme::{self, LevelFilter};
use fnv::FnvHashMap as HashMap;
use generational_arena::{Arena, Index};
use log;
use std::io::{ErrorKind, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use fennel::commands::look;
use fennel::util::take_command;
use fennel::{
    define_commands, listen, lookup_command, Area, CharId, Character, Connection,
    ConnectionBuilder, PlayerRecord, Room, RoomId,
};

static PULSE_PER_SECOND: u32 = 3;
static PULSE_RATE_NS: u32 = 1_000_000_000 / PULSE_PER_SECOND;

fn audit_room_exits(rooms: &mut HashMap<RoomId, Room>) {
    // I should be able to do this in a single iteration of room.values_mut(),
    // and I'm angry that I can't.
    let mut destinations_to_remove = vec![];
    for room in rooms.values() {
        for exit in room.exits.as_ref() {
            if !rooms.contains_key(&exit.to) {
                destinations_to_remove.push(exit.to)
            }
        }
    }
    if !destinations_to_remove.is_empty() {
        for room in rooms.values_mut() {
            for (n, exit) in room.exits.clone().as_ref().iter().enumerate() {
                if destinations_to_remove.contains(&exit.to) {
                    log::warn!(
                        "Loading areas: removed room {}'s exit '{}' to nonexistant {}",
                        room.id,
                        &exit.dir,
                        &exit.to
                    );
                    room.exits.remove(n);
                }
            }
        }
    }
}

fn load_areas() -> (Vec<Area>, HashMap<CharId, Character>, HashMap<RoomId, Room>) {
    log::info!("Loading areas");
    let mut areas = Vec::new();
    let mut rooms = HashMap::default();
    let mut npcs = HashMap::default();
    // FIXME: daaaaaaang this is ugly
    // TODO: we're gonna iterate over every area name in some text file, rather than load a single area
    match Area::load("default") {
        Ok(mut area_def) => {
            let area_npcs = area_def.extract_npcs();
            let room_defs = area_def.extract_rooms();

            let area = Area::from_prototype(area_def);
            let area_idx = areas.len();
            areas.push(area);
            let area = &mut areas[area_idx];

            for ch in area_npcs {
                if npcs.contains_key(&ch.id()) {
                    log::warn!("Loading areas: clobbered existing NPC {}", ch.id());
                }
                npcs.insert(ch.id(), ch);
            }

            for room_def in room_defs {
                let room = Room::from_prototype(room_def, area_idx);
                if rooms.contains_key(&room.id) {
                    log::warn!("Loading areas: clobbered existing room {}", room.id);
                }
                let room_idx = room.id;
                rooms.insert(room.id, room);
                area.rooms.push(room_idx);
            }

            audit_room_exits(&mut rooms);
        }
        Err(e) => log::error!("Error loading area {:?}", e),
    }
    log::info!("Loading areas: success");
    (areas, npcs, rooms)
}

fn accept_new_connections(
    connections: &mut Arena<Connection>,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
    receiver: &Receiver<(ConnectionBuilder, PlayerRecord)>,
) {
    while let Ok((conn_builder, record)) = receiver.try_recv() {
        let (player, char) = record.into_inner();

        let mut conn = if let Some((conn_index, _existing_conn)) = connections
            .iter()
            .find(|(_, c)| c.player_name() == player.name())
        {
            let existing_conn = connections.remove(conn_index).unwrap();
            log::info!(
                "Connection overridden from {} to {} for {}",
                existing_conn.addr(),
                conn_builder.addr,
                player.name()
            );
            conn_builder.logged_in(player, existing_conn.character)
        } else if let Some((char_idx, _char)) = characters
            .iter()
            .find(|(_, c)| c.keywords()[0] == player.name() && c.id() == Default::default())
        {
            log::info!(
                "Connection regained from {} for {}",
                conn_builder.addr,
                player.name()
            );
            let mut conn = conn_builder.logged_in(player, char_idx);
            let _ = write!(&mut conn, "Reconnecting...\n");
            conn
        } else {
            log::info!(
                "New connection from {} for {}",
                conn_builder.addr,
                player.name()
            );

            // Ensure that the character's room still exists.
            let mut char = char;
            if rooms.get(&char.in_room).is_none() {
                char.in_room = RoomId::default();
            }

            let in_room = room_chars
                .get_mut(&char.in_room)
                .expect("Unwrapped None room chars");
            let char_idx = characters.insert(char);
            in_room.push(char_idx);
            conn_builder.logged_in(player, char_idx)
        };

        let _ = look(&mut conn, "auto", characters, rooms, room_chars);
        let _conn_idx = connections.insert(conn);
    }
}

fn game_loop(
    connection_receiver: Receiver<(ConnectionBuilder, PlayerRecord)>,
) -> std::io::Result<()> {
    let mut last_time: Instant;
    let mut connections: Arena<Connection> = Arena::new();
    let mut characters: Arena<Character> = Arena::new();
    let mut mark_for_disconnect = Vec::new();

    let commands = define_commands();
    let (areas, npcs, rooms) = load_areas();
    let mut room_chars: HashMap<RoomId, Vec<Index>> = HashMap::default();
    for key in rooms.keys() {
        room_chars.insert(*key, vec![]);
    }

    for npc in npcs.values() {
        // FIXME: oh no! the area won't work when a mob disappears, we won't have the index to remove it D:
        let idx = characters.insert(npc.clone());
        let in_room = room_chars
            .get_mut(&npc.in_room)
            .expect("Unwrapped None room chars");
        in_room.push(idx);
    }

    println!("{:?}\n\n{:?}\n\n{:?}", areas, npcs, rooms);

    loop {
        last_time = Instant::now();

        // for (idx, conn) in &connections {
        //     if conn.character.is_none() {
        //         mark_for_disconnect.push(idx);
        //     }
        // }

        accept_new_connections(
            &mut connections,
            &mut characters,
            &rooms,
            &mut room_chars,
            &connection_receiver,
        );

        // handle input
        for (idx, conn) in &mut connections {
            match conn.read() {
                Ok(input) if !input.is_empty() => conn.input = Some(input),
                Ok(_) => {
                    log::debug!("Marking linkdead {}: zero length input", conn.addr());
                    mark_for_disconnect.push(idx);
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::ConnectionAborted | ErrorKind::ConnectionReset => {
                            log::debug!("Marking linkdead {}: {}", conn.addr(), e);
                            mark_for_disconnect.push(idx);
                        }
                        ErrorKind::WouldBlock => {} // explicitly okay no matter what happens to the catch-all branch
                        _ => log::warn!(
                            "Unexpected input read error from {}: {:?} {}",
                            conn.addr(),
                            e.kind(),
                            e
                        ),
                    }
                }
            }
            // decrement lag

            // interpret input
        }

        for idx in &mark_for_disconnect {
            if let Some(_conn) = connections.remove(*idx) {
                // TODO: close connection
            }
        }
        mark_for_disconnect.clear();

        // update world
        for (_idx, conn) in &mut connections {
            if let Some(input) = conn.input.take() {
                if let Some((command, rest)) = take_command(&input) {
                    let _ = if let Some(cmd) = lookup_command(&commands, command) {
                        cmd(conn, rest, &mut characters, &rooms, &mut room_chars)
                    } else {
                        write!(conn, "I have no idea what that means!")
                    };
                } else {
                    write!(conn, "\r\n");
                }
            }
        }

        // handle output
        for (_idx, conn) in &mut connections {
            let _ = conn.write_flush("You are who you are; You are where you are; The time is now>");
        }

        let now = Instant::now();
        let next_pulse = last_time + Duration::new(0, PULSE_RATE_NS);
        let sleep_for = if now < next_pulse {
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
