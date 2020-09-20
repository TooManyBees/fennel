#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Pronoun {
    It,
    He,
    She,
    They,
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
