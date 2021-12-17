use super::{Health, Human};
use crate::common::{map::Coord, thede};
use gardiz::{coord::Vec2, direc::Direction};
use std::{error::Error, fmt};

pub const MAX_HEALTH: Health = 20;

/// The ID of an NPC.
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

/// A handle to an NPC.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Npc {
    id: Id,
    data: NpcData,
}

impl Npc {
    pub fn new(id: Id, data: NpcData) -> Self {
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
pub struct NpcData {
    human: Human,
    thede: thede::Id,
}

impl NpcData {
    pub fn new(head: Vec2<Coord>, facing: Direction, thede: thede::Id) -> Self {
        Self {
            human: Human {
                max_health: MAX_HEALTH,
                health: MAX_HEALTH,
                head,
                facing,
            },
            thede,
        }
    }
}

/// Returned by [`Registry::load`] if the NPC does not exist.
#[derive(Debug, Clone, Copy)]
pub struct InvalidId(pub Id);

impl fmt::Display for InvalidId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Invalid NPC id {}", self.0)
    }
}

impl Error for InvalidId {}
