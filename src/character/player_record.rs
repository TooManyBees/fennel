use crate::character::{Character, CharacterData, Player, Pronoun};
use crate::object::Object;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerRecord {
    name: String,
    password: String,
    character: CharacterData,
    #[serde(default)]
    inventory: Vec<Object>,
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
            character: CharacterData::new_player(keywords, formal_name, pronoun),
            inventory: vec![],
        }
    }

    pub fn from_player(player: &Player, character: &Character) -> PlayerRecord {
        let inventory = character.inventory.iter().cloned().collect();
        PlayerRecord {
            name: player.name.clone(),
            password: player.password.clone(),
            character: character.data.clone(),
            inventory,
        }
    }

    pub fn name(self) -> String {
        self.name
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn into_inner(self) -> (Player, CharacterData, Vec<Object>) {
        let player = Player {
            name: self.name,
            password: self.password,
        };
        let character = self.character;
        let inventory = self.inventory;
        (player, character, inventory)
    }
}
