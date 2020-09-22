use serde::{Deserialize, Serialize};
use std::default::Default;

use crate::pronoun::Pronoun;
use crate::room::RoomId;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Character {
    name: String,
    pronoun: Pronoun,
    password: String,
    #[serde(default)]
    pub in_room: RoomId,
}

impl Character {
    pub fn new(name: String, pronoun: Pronoun, password: String) -> Character {
        Character {
            name,
            pronoun,
            password,
            // connection: None
            ..Default::default()
        }
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pronoun(&self) -> Pronoun {
        self.pronoun
    }
}
