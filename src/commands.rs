use crate::util::{self, take_argument};
use crate::{Character, Connection, PlayerRecord, Room, RoomId};
use fnv::FnvHashMap as HashMap;
use generational_arena::Arena;
use std::io::{Result as IoResult, Write};

// pub enum Position {
//     Asleep,
//     Resting,
//     Ready,
//     Fighting,
// }

pub type CommandFn =
    fn(&mut Connection, &str, &mut Arena<Character>, &HashMap<RoomId, Room>) -> IoResult<()>;
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
) -> IoResult<()> {
    let character = characters[conn.character].clone();
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
) -> IoResult<()> {
    let char = &characters[conn.character];
    let room = &rooms[&char.in_room];
    let (arg, _) = take_argument(arguments);
    // FIXME: Okay, we're absolutely going with intrusive lists for chars/objs/etc in the room. Just not now.
    match arg {
        Some("auto") | None => {
            write!(
                conn,
                "{}\n{}\n{}\n",
                &room.name, &room.exits, &room.description
            )?;
            for (_, ch) in characters
                .iter()
                .filter(|(_, ch)| ch.in_room == char.in_room)
            {
                write!(conn, "{}\n", ch.room_description())?;
            }
        }
        Some(a) => {
            if let Some((_, target)) = characters
                .iter()
                .find(|(_, ch)| ch.name().starts_with(a) && ch.in_room == char.in_room)
            {
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
) -> IoResult<()> {
    move_char(conn, "north", characters, rooms)
}

fn south(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
) -> IoResult<()> {
    move_char(conn, "south", characters, rooms)
}

fn east(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
) -> IoResult<()> {
    move_char(conn, "east", characters, rooms)
}

fn west(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
) -> IoResult<()> {
    move_char(conn, "west", characters, rooms)
}

fn move_char(
    conn: &mut Connection,
    arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
) -> IoResult<()> {
    let char = &mut characters[conn.character];
    if let Some(exit) = rooms
        .get(&char.in_room)
        .and_then(|room| room.exits.get(arguments))
    {
        let to_room = &rooms[&exit.to]; // We audited this

        // leave message to room
        char.in_room = to_room.id;
        // arrive message to room
        look(conn, "auto", characters, rooms)?;
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
