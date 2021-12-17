use super::{Health, Human};
use crate::common::map::Coord;
use gardiz::{coord::Vec2, direc::Direction};
use std::fmt;

const MAX_HEALTH: Health = 20;

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

impl Id {
    pub fn new(count: u32) -> Self {
        Self(count)
    }
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

impl PlayerData {
    pub fn new(head: Vec2<Coord>, facing: Direction) -> Self {
        Self {
            human: Human {
                max_health: MAX_HEALTH,
                health: MAX_HEALTH,
                head,
                facing,
            },
        }
    }
}
