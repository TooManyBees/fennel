use std::default::Default;
use std::fs::File;
use std::path::Path;

use crate::pronoun::Pronoun;

#[derive(Debug)]
pub struct Character {
    name: String,
    pronoun: Pronoun,
    password: String,
}

impl Character {
    pub fn load(name: &str) -> Result<Option<Character>, ()> {
        let path = Path::new("players").join(Path::new(name));
        unimplemented!()
    }

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
