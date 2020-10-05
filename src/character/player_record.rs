use crate::character::{Character, Player, Pronoun};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerRecord {
    name: String,
    password: String,
    character: Character,
}

impl PlayerRecord {
    pub fn file_path(name: &str) -> PathBuf {
        Path::new("players").join(name).with_extension("json")
    }

    pub fn new(name: String, pronoun: Pronoun, password: String) -> PlayerRecord {
        let formal_name = name.clone(); // TODO: this should be chosen on char chreation
        let keywords = vec![name.clone()];
        PlayerRecord {
            name,
            password,
            character: Character::new_player(keywords, formal_name, pronoun),
        }
    }

    pub fn from_player(player: Player, character: Character) -> PlayerRecord {
        PlayerRecord {
            name: player.name,
            password: player.password,
            character,
        }
    }

    pub fn name(self) -> String {
        self.name
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
