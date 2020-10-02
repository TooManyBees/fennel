use crate::character::{CharId, Character};
use crate::object::ObjectDef;
use crate::room::{RoomDef, RoomId};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct AreaDef {
    name: String,
    author: String,
    pub npcs: Vec<Character>,
    pub objects: Vec<ObjectDef>,
    pub rooms: Vec<RoomDef>,
}

impl AreaDef {
    pub fn extract_rooms(&mut self) -> Vec<RoomDef> {
        std::mem::take(&mut self.rooms)
    }

    pub fn extract_objects(&mut self) -> Vec<ObjectDef> {
        std::mem::take(&mut self.objects)
    }

    pub fn extract_npcs(&mut self) -> Vec<Character> {
        std::mem::take(&mut self.npcs)
    }
}

#[derive(Default, Debug)]
pub struct Area {
    name: String,
    author: String,
    age: u32,
    pub rooms: Vec<RoomId>,
}

#[derive(Debug)]
pub enum AreaLoadError {
    IO(std::io::Error),
    Parse(toml::de::Error),
}

impl From<std::io::Error> for AreaLoadError {
    fn from(e: std::io::Error) -> AreaLoadError {
        AreaLoadError::IO(e)
    }
}

impl From<toml::de::Error> for AreaLoadError {
    fn from(e: toml::de::Error) -> AreaLoadError {
        AreaLoadError::Parse(e)
    }
}

impl Area {
    pub fn load<P: AsRef<Path>>(name: P) -> Result<AreaDef, AreaLoadError> {
        let path = Path::new("areas").join(name).with_extension("toml");
        let mut s = String::new();
        let mut f = File::open(path)?;
        f.read_to_string(&mut s)?;
        let area: AreaDef = toml::from_str(&s)?;
        Ok(area)
    }

    pub fn from_prototype(area_def: AreaDef) -> Area {
        Area {
            name: area_def.name,
            author: area_def.author,
            rooms: Vec::with_capacity(area_def.rooms.len()),
            ..Default::default()
        }
    }
}
