use crate::util::{self, take_argument};
use crate::world::Recipient;
use crate::{PlayerRecord, RoomId, World};
use generational_arena::Index;
use std::io::{Result as IoResult, Write};

pub type CommandFn = fn(Index, &str, &mut World) -> IoResult<()>;

pub const COMMANDS: &[(&'static str, CommandFn)] = &[
    ("north", north),
    ("south", south),
    ("east", east),
    ("west", west),
    ("look", look),
    ("save", save),
    ("quit", quit),
];

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
        Err(e) => write!(
            conn,
            "Your character couldn't be saved.\r\nBailing from quit.\r\n"
        ),
    }
}

pub fn look(conn_idx: Index, arguments: &str, world: &mut World) -> IoResult<()> {
    let conn = world
        .connections
        .get_mut(conn_idx)
        .expect("Unwrapped None connection");
    let (arg, _) = take_argument(arguments);

    let room;
    let chars_in_room;
    let objs_in_room;
    {
        let char = world
            .characters
            .get(conn.character)
            .expect("Unwrapped None character");
        room = world.rooms.get(&char.in_room).expect("Unwrapped None room");
        chars_in_room = world
            .room_chars
            .get(&char.in_room)
            .expect("Unwrapped None room chars");
        objs_in_room = world
            .room_objs
            .get(&char.in_room)
            .expect("Unwrapped None room objs");
    };

    match arg {
        Some("auto") | None => {
            write!(
                conn,
                "{}\r\n{}\r\n{}\r\n",
                &room.name, &room.exits, &room.description
            )?;

            for obj in objs_in_room {
                write!(conn, "    {}\r\n", obj.room_description())?;
            }

            let self_idx = conn.character;
            for char_idx in chars_in_room {
                if *char_idx == self_idx {
                    continue;
                }
                if let Some(ch) = world.characters.get(*char_idx) {
                    write!(conn, "{}\r\n", ch.room_description())?;
                }
            }
            // for ch in chars_in_room.iter().copied().filter_map(|idx| {
            //     if idx != self_idx {
            //         world.characters.get(idx)
            //     } else {
            //         None
            //     }
            // }) {
            //     write!(conn, "{}\r\n", ch.room_description())?;
            // }
        }
        Some(a) => {
            for char_idx in chars_in_room {
                if let Some(target) = world
                    .characters
                    .get(*char_idx)
                    .filter(|ch| ch.keywords().iter().any(|kw| kw.starts_with(a)))
                {
                    write!(
                        conn,
                        "You look at {}.\r\n{}\r\n",
                        target.formal_name(),
                        target.description()
                    )?;
                    return Ok(());
                }
            }
            if let Some(target) = objs_in_room
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

fn north(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, "north", world)
}

fn south(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, "south", world)
}

fn east(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, "east", world)
}

fn west(conn_idx: Index, _arguments: &str, world: &mut World) -> IoResult<()> {
    move_char(conn_idx, "west", world)
}

// TODO: rename this to navigate_char, because it's using directions. move_char should be reserved
// for moving a character directly to any arbitrary room.
fn move_char(conn_idx: Index, arguments: &str, world: &mut World) -> IoResult<()> {
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
        .and_then(|room| room.exits.get(arguments))
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

    const COMMANDS: &[(&'static str, FakeCommand)] = &[
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
        assert_eq!(Some(&North), lookup_command(&COMMANDS, "north"));
    }

    #[test]
    fn prioritize_earlier_matches() {
        assert_eq!(Some(&North), lookup_command(&COMMANDS, "no"));
    }

    #[test]
    fn prioritize_exact_matches() {
        assert_eq!(Some(&Throw), lookup_command(&COMMANDS, "throw"));
    }

    #[test]
    fn find_partial_match() {
        assert_eq!(Some(&There), lookup_command(&COMMANDS, "the"));
    }
}
