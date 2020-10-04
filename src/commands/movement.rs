use super::informational::look;
use crate::util;
use crate::world::{Recipient, World};
use generational_arena::Index;
use std::io::{Result as IoResult, Write};

pub fn north(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, "north", world)
}

pub fn south(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, "south", world)
}

pub fn east(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, "east", world)
}

pub fn west(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, "west", world)
}

pub fn up(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, "up", world)
}

pub fn down(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, "down", world)
}

pub fn go(conn_idx: Index, arguments: &str, world: &mut World) -> IoResult<()> {
    let (direction, _) = util::take_argument(arguments);
    match direction {
        Some(direction) => move_char(conn_idx, direction, world),
        None => {
            let conn = world.connections.get_mut(conn_idx).unwrap();
            write!(conn, "Go where?\r\n")
        }
    }
}

fn move_char(conn_idx: Index, direction: &str, world: &mut World) -> IoResult<()> {
    let conn = world
        .connections
        .get(conn_idx)
        .expect("Unwrapped None connection");
    let (from_room, char_name) = {
        let char = world
            .characters
            .get(conn.character)
            .expect("Unwrapped None character");

        (char.in_room, char.formal_name().to_string())
    };
    if let Some(exit) = world
        .rooms
        .get(&from_room)
        .and_then(|room| room.exits.get(direction))
    {
        let to_room = world.rooms.get(&exit.to).expect("Unwrapped None room").id;
        let char_idx = conn.character;

        let leave_msg = format!("{} leaves {}.\r\n", char_name, exit.dir.leaving());
        let arrive_msg = format!("{} arrives from {}.\r\n", char_name, exit.dir.arriving());

        world.char_from_room(char_idx, from_room);
        world.msg_char(&leave_msg, Recipient::NotSubject(char_idx, from_room));
        world.msg_char(&arrive_msg, Recipient::NotSubject(char_idx, to_room));
        world.char_to_room(char_idx, to_room);

        // TODO: make a more fundamental "do_look" function that doesn't need to look up the room
        // first (since we already have it)
        look(conn_idx, "auto", world)?;
    } else {
        let conn = world.connections.get_mut(conn_idx).unwrap();
        write!(conn, "You can't go that way.\r\n")?;
    }
    Ok(())
}
