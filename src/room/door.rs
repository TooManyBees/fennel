use serde::{Deserialize, Serialize};

use std::default::Default;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Closeable {
    Closed,
    Open,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Lockable {
    Unlocked,
    Locked,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Door {
    None,
    Closable(Closeable),
    Lockable(Closeable, Lockable),
}

#[derive(Clone, Copy, Debug)]
pub enum DoorError {
    NoDoor,
    Closed,
    Opened,
    NoLock,
    Locked,
    Unlocked,
}

impl Default for Door {
    fn default() -> Self {
        Door::None
    }
}

impl Door {
    pub fn is_present(&self) -> bool {
        match self {
            Door::None => false,
            _ => true,
        }
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Door::None => false,
            Door::Closable(Closeable::Open) | Door::Lockable(Closeable::Open, _) => false,
            _ => true
        }
    }

    pub fn is_open(&self) -> bool {
        !self.is_closed()
    }

    pub fn open(&self) -> Result<Self, DoorError> {
        use Closeable::*;
        use Lockable::*;
        match self {
            Door::None => Err(DoorError::NoDoor),
            Door::Closable(Open) | Door::Lockable(Open, _) => Err(DoorError::Opened),
            Door::Closable(Closed) => Ok(Door::Closable(Open)),
            Door::Lockable(Closed, Unlocked) => Ok(Door::Lockable(Open, Unlocked)),
            Door::Lockable(_, Locked) => Err(DoorError::Locked),
        }
    }

    pub fn close(&self) -> Result <Self, DoorError> {
        use Closeable::*;
        use Lockable::*;
        match self {
            Door::None => Err(DoorError::NoDoor),
            Door::Closable(Open) => Ok(Door::Closable(Closed)),
            Door::Closable(Closed) | Door::Lockable(Closed, _) => Err(DoorError::Closed),
            Door::Lockable(Open, locked) => Ok(Door::Lockable(Closed, *locked)),
        }
    }

    pub fn lock(&self) -> Result<Self, DoorError> {
        use Closeable::*;
        use Lockable::*;
        match self {
            Door::None => Err(DoorError::NoDoor),
            Door::Closable(_) => Err(DoorError::NoLock),
            Door::Lockable(Closed, Locked) => Err(DoorError::Locked),
            Door::Lockable(Closed, Unlocked) => Ok(Door::Lockable(Closed, Locked)),
            Door::Lockable(Open, _) => Err(DoorError::Opened),
        }
    }

    pub fn unlock(&self) -> Result<Self, DoorError> {
        use Closeable::*;
        use Lockable::*;
        match self {
            Door::None => Err(DoorError::NoDoor),
            Door::Closable(_) => Err(DoorError::NoLock),
            Door::Lockable(Closed, Locked) => Ok(Door::Lockable(Closed, Unlocked)),
            Door::Lockable(Closed, Unlocked) => Err(DoorError::Unlocked),
            Door::Lockable(Open, _) => Err(DoorError::Opened),
        }
    }
}
