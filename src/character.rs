mod character_data;
mod player;
mod player_record;
mod pronoun;

use generational_arena::Index;
use intrusive_collections::LinkedList;
use serde::{Deserialize, Serialize};
use std::default::Default;

use crate::object::ObjectOnCharAdapter;
use crate::room::RoomId;
pub use character_data::CharacterData;
pub use player::Player;
pub use player_record::PlayerRecord;
pub use pronoun::Pronoun;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Default, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct CharId(u32);

impl Display for CharId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Default)]
pub struct Character {
    index: Option<Index>,
    connection: Option<Index>,
    data: CharacterData,
    pub inventory: LinkedList<ObjectOnCharAdapter>,
}

impl Character {
    pub fn from_data(char_data: CharacterData) -> Self {
        Character {
            data: char_data,
            ..Default::default()
        }
    }
    pub fn set_index(&mut self, index: Index) {
        self.index = Some(index);
    }

    pub fn index(&self) -> Option<Index> {
        self.index
    }

    pub fn set_connection(&mut self, index: Index) {
        self.connection = Some(index);
    }

    pub fn connection(&self) -> Option<Index> {
        self.connection
    }

    pub fn id(&self) -> CharId {
        self.data.id
    }

    pub fn keywords(&self) -> &[String] {
        &self.data.keywords
    }

    pub fn formal_name(&self) -> &str {
        &self.data.formal_name
    }

    pub fn description(&self) -> Description {
        Description {
            description: self.data.description.as_deref(),
            pronoun: self.data.pronoun,
        }
    }

    pub fn room_description(&self) -> RoomDescription {
        RoomDescription {
            room_description: self.data.room_description.as_deref(),
            name: &self
                .data
                .keywords
                .get(0)
                .expect("Missing first keyword for name"),
            formal_name: &self.data.formal_name,
        }
    }

    pub fn pronoun(&self) -> Pronoun {
        self.data.pronoun
    }

    pub fn in_room(&self) -> RoomId {
        self.data.in_room
    }

    pub fn set_in_room(&mut self, room_id: RoomId) {
        self.data.in_room = room_id;
    }
}

#[derive(Debug)]
pub struct Description<'ch> {
    description: Option<&'ch str>,
    pronoun: Pronoun,
}

impl<'ch> Display for Description<'ch> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(desc) = self.description {
            desc.fmt(f)
        } else {
            write!(
                f,
                "You don't see anything unusual about {}",
                self.pronoun.object()
            )
        }
    }
}

#[derive(Debug)]
pub struct RoomDescription<'ch> {
    room_description: Option<&'ch str>,
    name: &'ch str,
    formal_name: &'ch str,
    // position: Position,
}

impl<'ch> Display for RoomDescription<'ch> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(desc) = self.room_description {
            desc.fmt(f)
        } else {
            write!(f, "{} {{ {} }} is here.", self.formal_name, self.name)
        }
    }
}
