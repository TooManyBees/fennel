use crate::character::Pronoun;
use crate::room::RoomId;
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Copy, Clone, Debug, Default, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct NpcId(u32);

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CharacterData {
    #[serde(default, skip_serializing)]
    npc_id: NpcId,
    keywords: Vec<String>,
    formal_name: String,
    #[serde(skip_serializing)]
    room_description: Option<String>,
    description: Option<String>,
    pronoun: Pronoun,
    #[serde(default)]
    pub in_room: RoomId,
}
