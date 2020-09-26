use crate::{Character, Connection, Room, RoomId};
use fnv::FnvHashMap as HashMap;
use generational_arena::Arena;

// pub enum Position {
//     Asleep,
//     Resting,
//     Ready,
//     Fighting,
// }

pub type CommandFn = fn(&mut Connection, &str, &mut Arena<Character>, &HashMap<RoomId, Room>);
pub type CommandEntry = (String, CommandFn);

pub fn define_commands() -> Vec<CommandEntry> {
    vec![
        ("north".to_string(), north),
        ("south".to_string(), south),
        ("east".to_string(), east),
        ("west".to_string(), west),
        ("look".to_string(), look),
    ]
}

pub fn look(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
) {
    if let Some(char) = conn.character.and_then(|idx| characters.get(idx)) {
        let room = &rooms[&char.in_room];
        let _ = conn.write(&room.name);
        let _ = conn.write(&room.exits);
        let _ = conn.write(&room.description);
        // let _ = conn.write(&"");
    }
}

fn north(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
) {
    move_char(conn, "north", characters, rooms)
}

fn south(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
) {
    move_char(conn, "south", characters, rooms)
}

fn east(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
) {
    move_char(conn, "east", characters, rooms)
}

fn west(
    conn: &mut Connection,
    _arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
) {
    move_char(conn, "west", characters, rooms)
}

fn move_char(
    conn: &mut Connection,
    arguments: &str,
    characters: &mut Arena<Character>,
    rooms: &HashMap<RoomId, Room>,
) {
    let char = &mut characters[conn.character.unwrap()];
    if let Some(exit) = rooms
        .get(&char.in_room)
        .and_then(|room| room.exits.get(arguments))
    {
        let to_room = &rooms[&exit.to]; // We audited this
        // leave message to room
        char.in_room = to_room.id;
        // arrive message to room
        look(conn, "auto", characters, rooms);
    } else {
        let _ = conn.write(&"You can't go that way.");
    }
}

pub fn lookup_command<'a>(commands: &'a [CommandEntry], command: &str) -> Option<&'a CommandFn> {
    commands
        .iter()
        .find(|(name, _)| name.starts_with(&command))
        .map(|(_, cmd)| cmd)
}
