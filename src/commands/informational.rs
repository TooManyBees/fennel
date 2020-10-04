use crate::util;
use crate::world::World;
use generational_arena::Index;
use std::io::Result as IoResult;

pub fn look(conn_idx: Index, arguments: &str, world: &mut World) -> IoResult<()> {
    let conn = world
        .connections
        .get_mut(conn_idx)
        .expect("Unwrapped None connection");
    let (arg, _) = util::take_argument(arguments);
    let room_id = {
        let char = world
            .characters
            .get(conn.character)
            .expect("Unwrapped None character");
        char.in_room
    };

    match arg {
        Some("auto") | None => util::look_room(conn_idx, room_id, world),
        Some(target) => util::look_at(conn_idx, room_id, target, world),
    }
}
