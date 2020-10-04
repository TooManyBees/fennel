use crossbeam_channel::{bounded, Receiver};
use femme::{self, LevelFilter};
use log;
use std::io::Write;
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use fennel::{listen, util, ConnectionBuilder, PlayerRecord, RoomId, World};

static PULSE_PER_SECOND: u32 = 3;
static PULSE_RATE_NS: u32 = 1_000_000_000 / PULSE_PER_SECOND;

fn accept_new_connections(
    world: &mut World,
    receiver: &Receiver<(ConnectionBuilder, PlayerRecord)>,
) {
    while let Ok((conn_builder, record)) = receiver.try_recv() {
        let (player, char) = record.into_inner();

        let conn = if let Some((conn_index, _existing_conn)) = world
            .connections
            .iter()
            .find(|(_, c)| c.player_name() == player.name())
        {
            let existing_conn = world.connections.remove(conn_index).unwrap();
            log::info!(
                "Connection overridden from {} to {} for {}",
                existing_conn.addr(),
                conn_builder.addr,
                player.name()
            );
            conn_builder.logged_in(player, existing_conn.character)
        } else if let Some((char_idx, _char)) = world
            .characters
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
            if world.rooms.get(&char.in_room).is_none() {
                char.in_room = RoomId::default();
            }

            // TODO: use world.char_to_room here for consistency
            let in_room = world
                .room_chars
                .get_mut(&char.in_room)
                .expect("Unwrapped None room chars");
            let char_idx = world.characters.insert(char);
            world
                .characters
                .get_mut(char_idx)
                .map(|char| char.set_index(char_idx));
            in_room.push(char_idx);
            conn_builder.logged_in(player, char_idx)
        };

        // The actual char might be the one returned from the login thread, or one that was
        // taken over from a reconnection.
        let actual_char = world
            .characters
            .get_mut(conn.character)
            .expect("Unwrapped None character");
        let conn_idx = world.connections.insert(conn);
        actual_char.set_connection(conn_idx);
        let _ = util::look_room(conn_idx, actual_char.in_room, world);
    }
}

fn game_loop(
    connection_receiver: Receiver<(ConnectionBuilder, PlayerRecord)>,
) -> std::io::Result<()> {
    let mut last_time: Instant;

    let mut world = World::new();

    world.populate();

    loop {
        last_time = Instant::now();

        accept_new_connections(&mut world, &connection_receiver);

        // TODO: decrement lag here

        world.read_input();

        world.run_player_commands();

        // handle output
        for (_idx, conn) in &mut world.connections {
            let _ = conn.write_flush(Some(
                "You are who you are; You are where you are; The time is now>",
            ));
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
