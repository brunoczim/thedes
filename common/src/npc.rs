use crate::{block::Block, health::Health, human, thede, language};
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
    pub body: human::Body,
    pub health: Health,
    pub max_health: Health,
    pub thede: thede::Id,
    pub language: language::Id,
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
pub struct Npc {
    pub id: Id,
    pub data: Data,
}

impl Npc {
    pub fn block(&self) -> Block {
        Block::Npc(self.id)
    }
}
