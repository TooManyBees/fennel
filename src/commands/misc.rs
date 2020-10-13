use crate::character::PlayerRecord;
use crate::room::RoomId;
use crate::util;
use crate::world::{Recipient, World};
use generational_arena::Index;
use std::io::{Result as IoResult, Write};

pub fn save(
    conn_idx: Index,
    _room_id: RoomId,
    _arguments: &str,
    world: &mut World,
) -> IoResult<()> {
    let conn = world
        .connections
        .get_mut(conn_idx)
        .expect("Unwrapped None connection");
    let character = world
        .characters
        .get(conn.character)
        .expect("Unwrapped None character")
        .clone();
    let player_record = PlayerRecord::from_player(conn.player(), character);
    match util::save(conn.player_name(), player_record) {
        Ok(()) => write!(conn, "Saved!\r\n"),
        Err(_) => write!(conn, "Your character couldn't be saved.\r\n"),
    }
}

pub fn quit(
    conn_idx: Index,
    _room_id: RoomId,
    _arguments: &str,
    world: &mut World,
) -> IoResult<()> {
    let mut conn = world
        .connections
        .remove(conn_idx)
        .expect("Unwrapped None connection");
    let character = world
        .characters
        .remove(conn.character)
        .expect("Unwrapped None character");
    let player_room = character.in_room();
    let formal_name = character.formal_name().to_string();
    let pronoun = character.pronoun();

    let player_record = PlayerRecord::from_player(conn.player(), &character);
    match util::save(conn.player_name(), player_record) {
        Ok(()) => {
            let _ = write!(conn, "Saved!\r\nGoodbye.\r\n");
            let _ = conn.write_flush(None);
            world.char_from_room(conn.character, player_room);

            world.msg_char(
                &format!(
                    "{} flickers and fades as Reality takes {}.",
                    formal_name,
                    pronoun.object()
                ),
                Recipient::All(player_room),
            );

            log::info!("Player quit {} from {}", conn.player_name(), conn.addr());

            Ok(())
        }
        Err(_) => write!(
            conn,
            "Your character couldn't be saved.\r\nBailing from quit.\r\n"
        ),
    }
}
