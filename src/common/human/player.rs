use super::Human;
use std::fmt;

/// The ID of a player.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Id(u32);

fn dummy_id() -> Id {
    Id(0)
}

impl fmt::Display for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:x}", self.0)
    }
}

/// A handle to the player.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Player {
    id: Id,
    data: PlayerData,
}

impl Player {
    pub fn new(id: Id, data: PlayerData) -> Self {
        Self { id, data }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct PlayerData {
    human: Human,
}
