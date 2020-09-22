use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::File;
use std::io::Read;

#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct RoomId(u32);

impl Default for RoomId {
    fn default() -> RoomId {
        RoomId(1)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoomDef {
    id: RoomId,
    name: String,
    description: String,
    // flags
    // sector type
    // exits
    // extra descs
}

#[derive(Debug, Default)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub description: String,
    pub area: usize,
    // flags
    // sector type
    // exits
    // extra descs
    // objects
    // characters
}

impl Room {
    pub fn from_prototype(room_def: RoomDef, area: usize) -> Room {
        Room {
            id: room_def.id,
            name: room_def.name,
            description: room_def.description,
            area,
            ..Default::default()
        }
    }
}
