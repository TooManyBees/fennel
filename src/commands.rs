use crate::util::{self, take_argument};
use crate::{Character, Connection, PlayerRecord, Room, RoomId};
use fnv::FnvHashMap as HashMap;
use generational_arena::{Arena, Index};
use std::io::{Result as IoResult, Write};

// pub enum Position {
//     Asleep,
//     Resting,
//     Ready,
//     Fighting,
// }

pub type CommandFn = fn(
    &mut Connection,
    &str,
    &mut Arena<Character>,
    &HashMap<RoomId, Room>,
    &mut HashMap<RoomId, Vec<Index>>,
) -> IoResult<()>;
pub type CommandEntry = (String, CommandFn);

pub fn define_commands() -> Vec<CommandEntry> {
    vec![
        ("north".to_string(), north),
        ("south".to_string(), south),
        ("east".to_string(), east),
        ("west".to_string(), west),
        ("look".to_string(), look),
        ("save".to_string(), save),
    ]
}

pub fn save(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    _rooms: &HashMap<RoomId, Room>,
    _room_chars: &mut HashMap<RoomId, Vec<Index>>,
) -> IoResult<()> {
    let character = characters
        .get(conn.character)
        .expect("Unwrapped None character")
        .clone();
    let player_record = PlayerRecord::from_player(conn.player().clone(), character);
    match util::save(conn.player_name(), player_record) {
        Ok(()) => write!(conn, "Saved!\n"),
        Err(e) => {
            log::error!("SAVE ERROR for {}: {}", &conn.player_name(), e);
            write!(conn, "Your character couldn't be saved.")
        }
    }
}

pub fn look(
    conn: &mut Connection,
    arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
) -> IoResult<()> {
    let char = characters
        .get(conn.character)
        .expect("Unwrapped None character");
    let room = rooms.get(&char.in_room).expect("Unwrapped None room");
    let (arg, _) = take_argument(arguments);
    let in_room = room_chars
        .get(&char.in_room)
        .expect("Unwrapped None room chars");
    match arg {
        Some("auto") | None => {
            write!(
                conn,
                "{}\n{}\n{}\n",
                &room.name, &room.exits, &room.description
            )?;
            let self_idx = conn.character;
            for ch in in_room.iter().filter_map(|&idx| {
                if idx != self_idx {
                    characters.get(idx)
                } else {
                    None
                }
            }) {
                write!(conn, "{}\n", ch.room_description())?;
            }
        }
        Some(a) => {
            if let Some(target) = in_room
                .iter()
                .filter_map(|idx| characters.get(*idx))
                .find(|ch| ch.name().starts_with(a))
            {
                // TODO: if self, "you look at yourself"...
                write!(
                    conn,
                    "You look at {}.\n{}\n",
                    target.formal_name(),
                    target.description()
                )?;
            } else {
                write!(conn, "You don't see any {} here.\n", a)?;
            }
        }
    }
    Ok(())
}

fn north(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
) -> IoResult<()> {
    move_char(conn, "north", characters, rooms, room_chars)
}

fn south(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
) -> IoResult<()> {
    move_char(conn, "south", characters, rooms, room_chars)
}

fn east(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
) -> IoResult<()> {
    move_char(conn, "east", characters, rooms, room_chars)
}

fn west(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
) -> IoResult<()> {
    move_char(conn, "west", characters, rooms, room_chars)
}

// TODO: rename this to navigate_char, because it's using directions. move_char should be reserved
// for moving a character directly to any arbitrary room.
fn move_char(
    conn: &mut Connection,
    arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
) -> IoResult<()> {
    let char = characters
        .get_mut(conn.character)
        .expect("Unwrapped None character");
    if let Some(exit) = rooms
        .get(&char.in_room)
        .and_then(|room| room.exits.get(arguments))
    {
        let to_room = rooms.get(&exit.to).expect("Unwrapped None room");

        transfer_char(conn.character, to_room.id, characters, room_chars);
        // TODO: make a more fundamental "do_look" function that doesn't need to look up the room
        // first (since we already have it)
        look(conn, "auto", characters, rooms, room_chars)?;
    } else {
        write!(conn, "You can't go {}.", arguments)?;
    }
    Ok(())
}

pub fn lookup_command<'a>(commands: &'a [CommandEntry], command: &str) -> Option<&'a CommandFn> {
    if command.is_empty() {
        return None;
    }
    let command = command.to_ascii_lowercase();
    commands
        .iter()
        .find(|(name, _)| name.starts_with(&command))
        .map(|(_, cmd)| cmd)
}

// TODO: This is not a command; move it to a different module eventually
pub fn transfer_char(
    index: Index,
    to_room: RoomId,
    characters: &mut Arena<Character>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
) -> Result<(), ()> {
    let char = characters.get_mut(index).expect("Unwrapped None character");
    let in_room = room_chars
        .get_mut(&char.in_room)
        .expect("Unwrapped None room chars");
    if let Some(i) = in_room
        .iter()
        .enumerate()
        .find(|(_, idx)| **idx == index)
        .map(|(i, _)| i)
    {
        in_room.remove(i);
    } else {
        log::warn!("transfer_char: couldn't remove char from {}", char.in_room);
    }
    char.in_room = to_room;
    // Add char index to new room
    if let Some(in_room) = room_chars.get_mut(&char.in_room) {
        in_room.push(index);
        Ok(())
    } else {
        log::warn!("transfer_char: couldn't move to {}", to_room);
        Err(())
    }
}
