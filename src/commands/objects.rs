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

pub fn get(conn_idx: Index, arguments: &str, world: &mut World) -> IoResult<()> {
    let char_idx = world.connections.get(conn_idx).unwrap().character;
    let room_id = world.characters.get(char_idx).unwrap().in_room();
    let room_objs = world.room_objs.get_mut(&room_id).unwrap();
    match try_get(room_objs, arguments) {
        Ok(obj) => {
            // OOF!
            let char = world.characters.get(char_idx).unwrap();
            let char_name = char.formal_name().to_string();
            world.msg_char(
                &format!("You get {}.\r\n", obj.name()),
                Recipient::Subject(char_idx),
            );
            world.msg_char(
                &format!("{} gets {}.\r\n", char_name, obj.name()),
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

pub fn take(conn_idx: Index, arguments: &str, world: &mut World) -> IoResult<()> {
    // TODO: take is also to take from players
    get(conn_idx, arguments, world)
}

fn try_drop(
    inv: &mut LinkedList<ObjectOnCharAdapter>,
    arguments: &str,
) -> Result<Rc<Object>, &'static str> {
    let (keyword, _) = util::take_argument(arguments);
    let keyword = keyword.ok_or("Drop what?\r\n")?;
    util::pluck_item_from_list(inv, keyword).ok_or("You aren't carrying that.\r\n")
}

pub fn drop(conn_idx: Index, arguments: &str, world: &mut World) -> IoResult<()> {
    let char_idx = world.connections.get(conn_idx).unwrap().character;
    let char = world.characters.get_mut(char_idx).unwrap();

    match try_drop(&mut char.inventory, arguments) {
        Ok(obj) => {
            let char_name = char.formal_name().to_string();
            let room_id = char.in_room();
            world.msg_char(
                &format!("You drop {}.\r\n", obj.name()),
                Recipient::Subject(char_idx),
            );
            world.msg_char(
                &format!("{} drops {}.\r\n", char_name, obj.name()),
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
