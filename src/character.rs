use std::default::Default;
use std::fs::File;
use std::path::Path;

use crate::pronoun::Pronoun;

#[derive(Debug)]
pub struct Character {
    name: String,
    pronoun: Pronoun,
}

impl Character {
    pub fn load(name: &str) -> Result<Option<Character>, ()> {
        let path = Path::new("players").join(Path::new(name));
        unimplemented!()
    }

    pub fn new(name: String, pronoun: Pronoun) -> Character {
        Character { name, pronoun }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pronoun(&self) -> Pronoun {
        self.pronoun
    }
}
