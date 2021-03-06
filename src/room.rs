mod direction;
mod door;
mod exit;

use crate::object::ObjectId;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fmt::{Display, Formatter};

pub use direction::Direction;
pub use door::{Door, DoorError};
pub use exit::{Exit, Exits};

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
#[serde(rename_all = "kebab-case")]
pub struct RoomDef {
    id: RoomId,
    name: String,
    description: String,
    exits: Exits,
    #[serde(default)]
    load_objects: Vec<ObjectId>,
    // flags
    // sector type
    // extra descs
}

#[derive(Debug, Default)]
pub struct Room {
    pub id: RoomId,
    pub area: usize,
    pub name: String,
    pub description: String,
    pub exits: Exits,
    pub object_ids: Vec<ObjectId>, // FIXME: this is not real, it's for testing
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
            description: room_def.description.trim().to_string(),
            exits: room_def.exits,
            object_ids: room_def.load_objects,
            area,
            ..Default::default()
        }
    }
}
