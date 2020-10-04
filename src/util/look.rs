use crate::room::RoomId;
use crate::world::World;
use generational_arena::Index;
use std::io::{Result as IoResult, Write};

pub fn look_room(conn_idx: Index, room_id: RoomId, world: &mut World) -> IoResult<()> {
    let conn = world.connections.get_mut(conn_idx).unwrap();
    let room;
    let chars_in_room;
    let objs_in_room;
    {
        room = world.rooms.get(&room_id).expect("Unwrapped None room");
        chars_in_room = world
            .room_chars
            .get(&room_id)
            .expect("Unwrapped None room chars");
        objs_in_room = world
            .room_objs
            .get(&room_id)
            .expect("Unwrapped None room objs");
    };

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
    Ok(())
}

pub fn look_at(conn_idx: Index, room_id: RoomId, target: &str, world: &mut World) -> IoResult<()> {
    let conn = world.connections.get_mut(conn_idx).unwrap();
    let chars_in_room;
    let objs_in_room;
    {
        chars_in_room = world
            .room_chars
            .get(&room_id)
            .expect("Unwrapped None room chars");
        objs_in_room = world
            .room_objs
            .get(&room_id)
            .expect("Unwrapped None room objs");
    };

    for char_idx in chars_in_room {
        if let Some(target) = world
            .characters
            .get(*char_idx)
            .filter(|ch| ch.keywords().iter().any(|kw| kw.starts_with(target)))
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
        .find(|obj| obj.keywords().iter().any(|kw| kw.starts_with(target)))
    {
        write!(conn, "{}\r\n", target.description())?;
    } else {
        write!(conn, "You don't see any {} here.\r\n", target)?;
    }
    Ok(())
}
