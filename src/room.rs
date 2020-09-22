use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::File;
use std::io::Read;

pub type RoomId = u32;

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
    name: String,
    description: String,
    area: usize,
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
