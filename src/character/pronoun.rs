use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Pronoun {
    It,
    He,
    She,
    They,
}

impl Default for Pronoun {
    fn default() -> Pronoun {
        Pronoun::They
    }
}

impl Pronoun {
    pub fn subject(&self) -> &'static str {
        match self {
            Pronoun::It => "it",
            Pronoun::He => "he",
            Pronoun::She => "she",
            Pronoun::They => "they",
        }
    }

    pub fn object(&self) -> &'static str {
        match self {
            Pronoun::It => "it",
            Pronoun::He => "him",
            Pronoun::She => "her",
            Pronoun::They => "them",
        }
    }

    pub fn possessive(&self) -> &'static str {
        match self {
            Pronoun::It => "its",
            Pronoun::He => "his",
            Pronoun::She => "hers",
            Pronoun::They => "their",
        }
    }
}
