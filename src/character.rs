use serde::{Serialize, Deserialize};

use crate::pronoun::Pronoun;

#[derive(Debug, Serialize, Deserialize)]
pub struct Character {
    name: String,
    pronoun: Pronoun,
    password: String,
}

impl Character {
    pub fn new(name: String, pronoun: Pronoun, password: String) -> Character {
        Character { name, pronoun, password }
    }

    pub fn password(&self) -> &str {  &self.password }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pronoun(&self) -> Pronoun {
        self.pronoun
    }
}
