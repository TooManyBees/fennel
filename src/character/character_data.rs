use crate::character::{CharId, Pronoun};
use crate::room::RoomId;
use serde::{Deserialize, Serialize};
use std::default::Default;

// #[derive(Copy, Clone, Debug, Default, Deserialize, Hash, Eq, PartialEq, Serialize)]
// pub struct NpcId(u32);

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CharacterData {
    #[serde(default, skip_serializing)]
    pub(super) id: CharId,
    pub(super) keywords: Vec<String>,
    pub(super) formal_name: String,
    #[serde(skip_serializing)]
    pub(super) room_description: Option<String>,
    pub(super) description: Option<String>,
    pub(super) pronoun: Pronoun,
    #[serde(default)]
    pub(super) in_room: RoomId,
}

impl CharacterData {
    pub fn new_player(keywords: Vec<String>, formal_name: String, pronoun: Pronoun) -> Self {
        CharacterData {
            keywords,
            formal_name,
            pronoun,
            ..Default::default()
        }
    }

    pub fn id(&self) -> CharId {
        self.id
    }
}
