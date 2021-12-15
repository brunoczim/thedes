use super::human::{npc, player};

/// Kind of a block.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Block {
    /// Empty.
    Empty,
    /// Wall block.
    Wall,
    ///Small twigs for tools.
    Twig,
    Player(player::Id),
    Npc(npc::Id),
}
