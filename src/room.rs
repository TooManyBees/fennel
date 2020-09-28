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
    exits: Exits,
    // flags
    // sector type
    // extra descs
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Exit {
    pub to: RoomId,
    pub dir: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Exits(pub Vec<Exit>);

impl Exits {
    pub fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }

    pub fn get(&self, direction: &str) -> Option<&Exit> {
        self.0.iter().find(|exit| exit.dir.as_str() == direction)
    }
}

impl AsRef<Vec<Exit>> for Exits {
    fn as_ref(&self) -> &Vec<Exit> {
        &self.0
    }
}

impl Display for Exits {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "[Exits:")?;
        for exit in &self.0 {
            write!(f, " {}", exit.dir)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Room {
    pub id: RoomId,
    pub area: usize,
    pub name: String,
    pub description: String,
    pub exits: Exits,
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
            area,
            ..Default::default()
        }
    }
}
