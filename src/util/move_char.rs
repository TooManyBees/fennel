use crate::{Character, RoomChars, RoomId};
use generational_arena::Index;

pub enum FromRoomError {
    NoSuchRoom(RoomId),
    CharNotInRoom(Index, RoomId),
}

pub enum ToRoomError {
    NoSuchRoom(RoomId),
}

pub fn char_from_room(
    from_room: RoomId,
    char_idx: Index,
    room_chars: &mut RoomChars,
) -> Result<(), FromRoomError> {
    let in_room = room_chars
        .get_mut(&from_room)
        .ok_or(FromRoomError::NoSuchRoom(from_room))?;
    let (in_room_idx, _) = in_room
        .iter()
        .enumerate()
        .find(|(_, idx)| **idx == char_idx)
        .ok_or(FromRoomError::CharNotInRoom(char_idx, from_room))?;
    in_room.remove(in_room_idx);
    Ok(())
}

pub fn char_to_room(
    to_room: RoomId,
    char: &mut Character,
    room_chars: &mut RoomChars,
) -> Result<(), ToRoomError> {
    let in_room = room_chars
        .get_mut(&to_room)
        .ok_or(ToRoomError::NoSuchRoom(to_room))?;
    in_room.push(char.index().unwrap());
    char.set_in_room(to_room);
    Ok(())
}
