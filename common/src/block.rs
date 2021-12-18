use crate::{npc, player};

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
    Unknown,
    Empty,
    Wall,
    Twig,
    Player(player::Id),
    Npc(npc::Id),
}
