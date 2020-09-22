use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct RoomId(u32);

impl Default for RoomId {
    fn default() -> RoomId {
        RoomId(1)
    }
}

impl Display for RoomId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoomDef {
    id: RoomId,
    name: String,
    description: String,
    exits: Vec<Exit>,
    // flags
    // sector type
    // extra descs
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Exit {
    pub to: RoomId,
    pub dir: String,
}

#[derive(Debug, Default)]
pub struct Room {
    pub id: RoomId,
    pub area: usize,
    pub name: String,
    pub description: String,
    pub exits: Vec<Exit>,
    // flags
    // sector type
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
            exits: room_def.exits,
            area,
            ..Default::default()
        }
    }
}
