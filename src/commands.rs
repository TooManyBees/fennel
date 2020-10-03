use crate::util::{self, take_argument};
use crate::{Character, Connection, ObjectInRoomAdapter, PlayerRecord, Room, RoomId};
use fnv::FnvHashMap as HashMap;
use generational_arena::{Arena, Index};
use intrusive_collections::LinkedList;
use std::io::{Result as IoResult, Write};

pub type CommandFn = fn(
    &mut Connection,
    &str,
    &mut Arena<Character>,
    &HashMap<RoomId, Room>,
    &mut HashMap<RoomId, Vec<Index>>,
    &mut HashMap<RoomId, LinkedList<ObjectInRoomAdapter>>,
) -> IoResult<()>;

pub fn define_commands() -> Vec<(&'static str, CommandFn)> {
    vec![
        ("north", north),
        ("south", south),
        ("east", east),
        ("west", west),
        ("look", look),
        ("save", save),
    ]
}

pub fn save(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    _rooms: &HashMap<RoomId, Room>,
    _room_chars: &mut HashMap<RoomId, Vec<Index>>,
    _room_objs: &mut HashMap<RoomId, LinkedList<ObjectInRoomAdapter>>,
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
    room_objs: &mut HashMap<RoomId, LinkedList<ObjectInRoomAdapter>>,
) -> IoResult<()> {
    let char = characters
        .get(conn.character)
        .expect("Unwrapped None character");
    let room = rooms.get(&char.in_room).expect("Unwrapped None room");
    let (arg, _) = take_argument(arguments);
    let chars_in_room = room_chars
        .get(&char.in_room)
        .expect("Unwrapped None room chars");
    let objs_in_room = room_objs
        .get(&char.in_room)
        .expect("Unwrapped None room objs");

    match arg {
        Some("auto") | None => {
            write!(
                conn,
                "{}\r\n{}\r\n{}\r\n",
                &room.name, &room.exits, &room.description
            )?;

            for obj in objs_in_room {
                write!(conn, "    {}\r\n", obj.room_description());
            }

            let self_idx = conn.character;
            for ch in chars_in_room.iter().filter_map(|&idx| {
                if idx != self_idx {
                    characters.get(idx)
                } else {
                    None
                }
            }) {
                write!(conn, "{}\r\n", ch.room_description())?;
            }
        }
        Some(a) => {
            if let Some(target) = chars_in_room
                .iter()
                .filter_map(|idx| characters.get(*idx))
                .find(|ch| ch.keywords().iter().any(|kw| kw.starts_with(a)))
            {
                // TODO: if self, "you look at yourself"...
                write!(
                    conn,
                    "You look at {}.\r\n{}\r\n",
                    target.formal_name(),
                    target.description()
                )?;
            } else if let Some(target) = objs_in_room
                .iter()
                .find(|obj| obj.keywords().iter().any(|kw| kw.starts_with(a)))
            {
                write!(conn, "{}\r\n", target.description())?;
            } else {
                write!(conn, "You don't see any {} here.\r\n", a)?;
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
    room_objs: &mut HashMap<RoomId, LinkedList<ObjectInRoomAdapter>>,
) -> IoResult<()> {
    move_char(conn, "north", characters, rooms, room_chars, room_objs)
}

fn south(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
    room_objs: &mut HashMap<RoomId, LinkedList<ObjectInRoomAdapter>>,
) -> IoResult<()> {
    move_char(conn, "south", characters, rooms, room_chars, room_objs)
}

fn east(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
    room_objs: &mut HashMap<RoomId, LinkedList<ObjectInRoomAdapter>>,
) -> IoResult<()> {
    move_char(conn, "east", characters, rooms, room_chars, room_objs)
}

fn west(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
    room_objs: &mut HashMap<RoomId, LinkedList<ObjectInRoomAdapter>>,
) -> IoResult<()> {
    move_char(conn, "west", characters, rooms, room_chars, room_objs)
}

// TODO: rename this to navigate_char, because it's using directions. move_char should be reserved
// for moving a character directly to any arbitrary room.
fn move_char(
    conn: &mut Connection,
    arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
    room_chars: &mut HashMap<RoomId, Vec<Index>>,
    room_objs: &mut HashMap<RoomId, LinkedList<ObjectInRoomAdapter>>,
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
        look(conn, "auto", characters, rooms, room_chars, room_objs)?;
    } else {
        write!(conn, "You can't go {}.\r\n", arguments)?;
    }
    Ok(())
}

pub fn lookup_command<'a, T>(commands: &'a [(&'static str, T)], command: &str) -> Option<&'a T> {
    if command.is_empty() {
        return None;
    }
    let command = command.to_ascii_lowercase();

    let mut found: Option<&'a T> = None;

    for (name, cmd_fn) in commands {
        let is_match = name.starts_with(&command);
        let exact = name.len() == command.len();
        if is_match && exact {
            found = Some(cmd_fn);
            break;
        }
        if is_match && found.is_none() {
            found = Some(cmd_fn);
        }
    }

    found
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

#[cfg(test)]
mod test {
    use super::lookup_command;

    #[derive(Debug, Eq, PartialEq)]
    enum FakeCommand {
        Nobody,
        North,
        Northern,
        There,
        Thorn,
        Throw,
        ThrowAway,
    }
    use FakeCommand::*;

    const commands: &[(&'static str, FakeCommand)] = &[
        ("north", North),
        ("northern", Northern),
        ("throwaway", ThrowAway),
        ("throw", Throw),
        ("nobody", Nobody),
        ("thorn", Thorn),
        ("there", There),
    ];

    #[test]
    fn find_exact_command() {
        assert_eq!(Some(&North), lookup_command(&commands, "north"));
    }

    #[test]
    fn prioritize_earlier_matches() {
        assert_eq!(Some(&North), lookup_command(&commands, "no"));
    }

    #[test]
    fn prioritize_exact_matches() {
        assert_eq!(Some(&Throw), lookup_command(&commands, "throw"));
    }

    #[test]
    fn find_partial_match() {
        assert_eq!(Some(&There), lookup_command(&commands, "the"));
    }
}
