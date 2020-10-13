use super::informational::look;
use crate::room::RoomId;
use crate::util;
use crate::world::{Recipient, World};
use generational_arena::Index;
use std::io::{Result as IoResult, Write};

pub fn north(
    conn_idx: Index,
    at_room: RoomId,
    _arguments: &str,
    world: &mut World,
) -> IoResult<()> {
    move_char(conn_idx, at_room, "north", world)
}

pub fn south(
    conn_idx: Index,
    at_room: RoomId,
    _arguments: &str,
    world: &mut World,
) -> IoResult<()> {
    move_char(conn_idx, at_room, "south", world)
}

pub fn east(conn_idx: Index, at_room: RoomId, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, at_room, "east", world)
}

pub fn west(conn_idx: Index, at_room: RoomId, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, at_room, "west", world)
}

pub fn up(conn_idx: Index, at_room: RoomId, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, at_room, "up", world)
}

pub fn down(conn_idx: Index, at_room: RoomId, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, at_room, "down", world)
}

pub fn go(conn_idx: Index, at_room: RoomId, arguments: &str, world: &mut World) -> IoResult<()> {
    let (direction, _) = util::take_argument(arguments);
    match direction {
        Some(direction) => move_char(conn_idx, at_room, direction, world),
        None => {
            let conn = world.connections.get_mut(conn_idx).unwrap();
            write!(conn, "Go where?\r\n")
        }
    }
}

fn move_char(conn_idx: Index, at_room: RoomId, direction: &str, world: &mut World) -> IoResult<()> {
    let conn = world
        .connections
        .get(conn_idx)
        .expect("Unwrapped None connection");
    let from_room = at_room;
    let char_name = world
        .characters
        .get(conn.character)
        .expect("Unwrapped None character")
        .formal_name()
        .to_string();

    if let Some(exit) = world
        .rooms
        .get(&from_room)
        .and_then(|room| room.exits.get(direction))
    {
        let to_room = world.rooms.get(&exit.to).expect("Unwrapped None room").id;
        let char_idx = conn.character;

        let leave_msg = format!("{} leaves {}.", char_name, exit.dir.leaving());
        let arrive_msg = format!("{} arrives from {}.", char_name, exit.dir.arriving());

        world.char_from_room(char_idx, from_room);
        world.msg_char(&leave_msg, Recipient::NotSubject(char_idx, from_room));
        world.msg_char(&arrive_msg, Recipient::NotSubject(char_idx, to_room));
        world.char_to_room(char_idx, to_room);

        util::look_room(conn_idx, to_room, world)?;
    } else {
        let conn = world.connections.get_mut(conn_idx).unwrap();
        write!(conn, "You can't go that way.\r\n")?;
    }
    Ok(())
}
