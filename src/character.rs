use serde::{Deserialize, Serialize};
use std::default::Default;

use crate::pronoun::Pronoun;
use crate::room::RoomId;

#[derive(Copy, Clone, Debug, Default, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct CharId(u32);

#[derive(Debug)]
pub struct Player {
    name: String,
    password: String,
}

impl Player {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerRecord {
    name: String,
    password: String,
    character: Character,
}

impl PlayerRecord {
    pub fn new(name: String, pronoun: Pronoun, password: String) -> PlayerRecord {
        PlayerRecord {
            name: name.clone(),
            password,
            character: Character {
                name: name.clone(),
                formal_name: name.clone(),
                room_description: name, // FIXME: %n is [standing|resting|sleeping] here
                pronoun,
                ..Default::default()
            },
        }
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn into_inner(self) -> (Player, Character) {
        let player = Player {
            name: self.name,
            password: self.password,
        };
        let character = self.character;
        (player, character)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Character {
    #[serde(default, skip_serializing)]
    id: CharId,
    name: String,
    #[serde(rename = "formal-name")]
    formal_name: String,
    #[serde(rename = "room-description")]
    room_description: String,
    #[serde(default)]
    description: String,
    pronoun: Pronoun,
    // password: String,
    #[serde(default)]
    pub in_room: RoomId,
}

impl Character {
    pub fn id(&self) -> CharId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pronoun(&self) -> Pronoun {
        self.pronoun
    }
}
