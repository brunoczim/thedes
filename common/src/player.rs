use crate::{block::Block, health::Health, map::Coord};
use gardiz::{coord::Vec2, direc::Direction};
use std::fmt;

pub const MAX_HEALTH: Health = Health { level: 20 };

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Id(pub u32);

impl fmt::Display for Id {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, fmt)
    }
}

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
pub struct Data {
    pub head: Vec2<Coord>,
    pub facing: Direction,
    pub health: Health,
    pub max_health: Health,
}

impl Data {
    #[inline]
    pub fn pointer(&self) -> Vec2<Coord> {
        self.head.move_one(self.facing)
    }
}

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
pub struct Player {
    pub id: Id,
    pub data: Data,
}

impl Player {
    pub fn block(&self) -> Block {
        Block::Player(self.id)
    }
}
