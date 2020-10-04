use super::{Direction, RoomId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Exit {
    pub to: RoomId,
    pub dir: Direction,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Exits(pub Vec<Exit>);

impl Exits {
    pub fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }

    pub fn get(&self, direction: &str) -> Option<&Exit> {
        self.0.iter().find(|exit| exit.dir.matches(direction))
    }
}

impl AsRef<Vec<Exit>> for Exits {
    fn as_ref(&self) -> &Vec<Exit> {
        &self.0
    }
}

impl Display for Exits {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "[Exits:")?;
        for exit in &self.0 {
            write!(f, " {}", exit.dir.leaving())?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
