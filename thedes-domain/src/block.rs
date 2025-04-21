use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[repr(u8)]
pub enum Block {
    Placeable(PlaceableBlock),
    Special(SpecialBlock),
}

impl Default for Block {
    fn default() -> Self {
        Self::Placeable(PlaceableBlock::default())
    }
}

impl From<PlaceableBlock> for Block {
    fn from(block: PlaceableBlock) -> Self {
        Self::Placeable(block)
    }
}

impl From<SpecialBlock> for Block {
    fn from(block: SpecialBlock) -> Self {
        Self::Special(block)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
)]
#[repr(u8)]
pub enum PlaceableBlock {
    #[default]
    Air = 0,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
)]
#[repr(u8)]
pub enum SpecialBlock {
    #[default]
    Player = 0,
}
