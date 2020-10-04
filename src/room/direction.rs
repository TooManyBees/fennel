use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
    Custom {
        to_name: String,
        from_name: String,
        keywords: Vec<String>,
    },
}

impl Direction {
    pub fn leaving(&self) -> &str {
        match self {
            Direction::North => "north",
            Direction::South => "south",
            Direction::East => "east",
            Direction::West => "west",
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Custom { to_name, .. } => to_name.as_str(),
        }
    }

    pub fn arriving(&self) -> &str {
        match self {
            Direction::North => "the south",
            Direction::South => "the north",
            Direction::East => "the west",
            Direction::West => "the east",
            Direction::Up => "below",
            Direction::Down => "above",
            Direction::Custom { from_name, .. } => from_name.as_str(),
        }
    }

    pub fn matches(&self, command: &str) -> bool {
        // TODO: nsewud directions should also be able to have their own keywords
        match self {
            Direction::North => "north".starts_with(command),
            Direction::South => "south".starts_with(command),
            Direction::East => "east".starts_with(command),
            Direction::West => "west".starts_with(command),
            Direction::Up => "up".starts_with(command),
            Direction::Down => "down".starts_with(command),
            Direction::Custom { keywords, .. } => keywords.iter().any(|kw| kw.starts_with(command)),
        }
    }
}
