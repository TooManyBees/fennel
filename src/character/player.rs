use crate::character::{Character, Pronoun};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Player {
    pub(super) name: String,
    pub(super) password: String,
}

impl Player {
    pub fn name(&self) -> &str {
        &self.name
    }
}
