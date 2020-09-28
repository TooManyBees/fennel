use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::{Path, PathBuf};

use crate::pronoun::Pronoun;
use crate::room::RoomId;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Default, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct CharId(u32);

impl Display for CharId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
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
    pub fn file_path(name: &str) -> PathBuf {
        Path::new("players").join(name).with_extension("json")
    }

    pub fn new(name: String, pronoun: Pronoun, password: String) -> PlayerRecord {
        let formal_name = name.clone();
        let keywords = vec![name.clone()];
        PlayerRecord {
            name,
            password,
            character: Character {
                keywords,
                formal_name,
                pronoun,
                ..Default::default()
            },
        }
    }

    pub fn from_player(player: Player, character: Character) -> PlayerRecord {
        PlayerRecord {
            name: player.name,
            password: player.password,
            character,
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Character {
    #[serde(default, skip_serializing)]
    id: CharId,
    keywords: Vec<String>,
    formal_name: String,
    #[serde(skip_serializing)]
    room_description: Option<String>,
    description: Option<String>,
    pronoun: Pronoun,
    // password: String,
    #[serde(default)]
    pub in_room: RoomId,
}

impl Character {
    pub fn id(&self) -> CharId {
        self.id
    }

    pub fn keywords(&self) -> &[String] {
        &self.keywords
    }

    pub fn formal_name(&self) -> &str {
        &self.formal_name
    }

    pub fn description(&self) -> Description {
        Description {
            description: self.description.as_deref(),
            pronoun: self.pronoun,
        }
    }

    pub fn room_description(&self) -> RoomDescription {
        RoomDescription {
            room_description: self.room_description.as_deref(),
            name: &self
                .keywords
                .get(0)
                .expect("Missing first keyword for name"),
            formal_name: &self.formal_name,
        }
    }

    pub fn pronoun(&self) -> Pronoun {
        self.pronoun
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
            write!(f, "{} {{ {} }} is here.\r\n", self.formal_name, self.name)
        }
    }
}
