pub mod npc;
pub mod player;

use crate::common::map::Coord;
use gardiz::{coord::Vec2, direc::Direction};

pub type Health = u8;

/// A generic human entity.
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
struct Human {
    /// Coordinates of the head.
    head: Vec2<Coord>,
    /// The direction the human is facing.
    facing: Direction,
    /// The human health.
    health: Health,
    /// The human maximum health.
    max_health: Health,
}
