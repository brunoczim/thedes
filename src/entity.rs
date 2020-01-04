use crate::orient::{Coord2D, Direc};

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
/// An entity ID.
pub struct Id(u32);

impl Id {
    /// Player's ID.
    pub const PLAYER: Self = Self(0);
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
/// A player entity.
pub struct Player {
    center: Coord2D,
    facing: Direc,
}

impl Player {
    pub const INIT: Self = Self { center: Coord2D::ORIGIN, facing: Direc::Up };
}
