use generational_arena::Index;
use std::io::{Result as IoResult, Write};
use crate::world::World;
use crate::player::PlayerRecord;
use crate::util;

pub fn save(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    let conn = world
        .connections
        .get_mut(conn_idx)
        .expect("Unwrapped None connection");
    let character = world
        .characters
        .get(conn.character)
        .expect("Unwrapped None character")
        .clone();
    let player_record = PlayerRecord::from_player(conn.player().clone(), character);
    match util::save(conn.player_name(), player_record) {
        Ok(()) => write!(conn, "Saved!\r\n"),
        Err(_) => write!(conn, "Your character couldn't be saved.\r\n"),
    }
}

pub fn quit(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    let mut conn = world
        .connections
        .remove(conn_idx)
        .expect("Unwrapped None connection");
    let character = world
        .characters
        .remove(conn.character)
        .expect("Unwrapped None character");
    let player_room = character.in_room;

    let player_record = PlayerRecord::from_player(conn.player().clone(), character);
    match util::save(conn.player_name(), player_record) {
        Ok(()) => {
            let _ = write!(conn, "Saved!\r\nGoodbye.\r\n");
            let _ = conn.write_flush(None);
            world.char_from_room(conn.character, player_room);

            if let Some(room) = world.rooms.get(&player_room) {
                for char_idx in &world.room_chars[&room.id] {
                    // FIXME: chars need indices back to their connections too!
                    // message players in room that they quit
                }
            }

            log::info!("Player quit {} from {}", conn.player_name(), conn.addr());

            Ok(())
        }
        Err(_) => write!(
            conn,
            "Your character couldn't be saved.\r\nBailing from quit.\r\n"
        ),
    }
}
