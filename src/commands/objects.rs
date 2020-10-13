use crate::object::Object;
use crate::world::{Recipient, World};
use crate::{util, ObjectInRoomAdapter, ObjectOnCharAdapter, RoomId};
use generational_arena::Index;
use intrusive_collections::LinkedList;
use std::io::{Result as IoResult, Write};
use std::rc::Rc;

fn try_get(
    room_objs: &mut LinkedList<ObjectInRoomAdapter>,
    arguments: &str,
) -> Result<Rc<Object>, &'static str> {
    let (keyword, _) = util::take_argument(arguments);
    let keyword = keyword.ok_or("Get what?\r\n")?;
    util::pluck_item_from_list(room_objs, keyword).ok_or("That isn't here.\r\n")
}

pub fn get(conn_idx: Index, room_id: RoomId, arguments: &str, world: &mut World) -> IoResult<()> {
    let char_idx = world.connections.get(conn_idx).unwrap().character;
    let room_objs = world.room_objs.get_mut(&room_id).unwrap();
    match try_get(room_objs, arguments) {
        Ok(obj) => {
            // OOF!
            let char = world.characters.get(char_idx).unwrap();
            let char_name = char.formal_name().to_string();
            world.msg_char(
                &format!("You get {}.", obj.name()),
                Recipient::Subject(char_idx),
            );
            world.msg_char(
                &format!("{} gets {}.", char_name, obj.name()),
                Recipient::NotSubject(char_idx, room_id),
            );
            let char = world.characters.get_mut(char_idx).unwrap();
            char.inventory.push_front(obj);
        }
        Err(e) => {
            let conn = world.connections.get_mut(conn_idx).unwrap();
            let _ = Write::write(conn, e.as_bytes());
        }
    }

    Ok(())
}

pub fn take(conn_idx: Index, room_id: RoomId, arguments: &str, world: &mut World) -> IoResult<()> {
    // TODO: take is also to take from players
    get(conn_idx, room_id, arguments, world)
}

fn try_drop(
    inv: &mut LinkedList<ObjectOnCharAdapter>,
    arguments: &str,
) -> Result<Rc<Object>, &'static str> {
    let (keyword, _) = util::take_argument(arguments);
    let keyword = keyword.ok_or("Drop what?\r\n")?;
    util::pluck_item_from_list(inv, keyword).ok_or("You aren't carrying that.\r\n")
}

pub fn drop(conn_idx: Index, room_id: RoomId, arguments: &str, world: &mut World) -> IoResult<()> {
    let char_idx = world.connections.get(conn_idx).unwrap().character;
    let char = world.characters.get_mut(char_idx).unwrap();

    match try_drop(&mut char.inventory, arguments) {
        Ok(obj) => {
            let char_name = char.formal_name().to_string();
            world.msg_char(
                &format!("You drop {}.", obj.name()),
                Recipient::Subject(char_idx),
            );
            world.msg_char(
                &format!("{} drops {}.", char_name, obj.name()),
                Recipient::NotSubject(char_idx, room_id),
            );
            let room_objs = world.room_objs.get_mut(&room_id).unwrap();
            room_objs.push_front(obj);
        }
        Err(e) => {
            let conn = world.connections.get_mut(conn_idx).unwrap();
            let _ = Write::write(conn, e.as_bytes());
        }
    }
    Ok(())
}

pub fn give(conn_idx: Index, room_id: RoomId, arguments: &str, world: &mut World) -> IoResult<()> {
    let (object_keyword, arguments) = util::take_argument(arguments);
    let (target_keyword, _) = util::take_argument(arguments);
    let char_idx = world.connections.get(conn_idx).unwrap().character;
    let room_chars = world.room_chars.get(&room_id).unwrap();

    let object_keyword = match object_keyword {
        Some(kw) => kw,
        None => {
            world.msg_char("Give what to whom?", Recipient::Subject(char_idx));
            return Ok(());
        }
    };
    let target_keyword = match target_keyword {
        Some(kw) => kw,
        None => {
            world.msg_char("Give it to whom?", Recipient::Subject(char_idx));
            return Ok(());
        }
    };

    let target_idx = room_chars.iter()
        .filter_map(|idx| world.characters.get(*idx))
        .find(|char| char.keywords().iter().any(|kw| kw.starts_with(target_keyword)))
        .and_then(|char| char.index());

    let target_idx = match target_idx {
        Some(idx) => idx,
        None => {
            world.msg_char("They aren't here.", Recipient::Subject(char_idx));
            return Ok(());
        }
    };

    if let Some(obj) = util::pluck_item_from_list(&mut world.characters.get_mut(char_idx).unwrap().inventory, object_keyword) {
        let source_char_name = world.characters.get(char_idx).unwrap().formal_name().to_string();
        let target_char = world.characters.get_mut(target_idx).unwrap();

        let char_message = format!("You give {} to {}.", obj.name(), target_char.formal_name());
        let target_message = format!("{} gives you {}.", source_char_name, obj.name());
        let room_message = format!("{} gives {} {}.", source_char_name, target_char.formal_name(), obj.name());

        target_char.inventory.push_front(obj);

        world.msg_char(&char_message, Recipient::Subject(char_idx));
        world.msg_char(&target_message, Recipient::Subject(target_idx));
        world.msg_char(&room_message, Recipient::Neither(char_idx, target_idx, room_id));
    } else {
        world.msg_char(&format!("You aren't holding any {} in your inventory.", object_keyword), Recipient::Subject(char_idx));
    }

    Ok(())
}
