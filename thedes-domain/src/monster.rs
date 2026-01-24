use serde::{Deserialize, Serialize};
use thedes_geometry::orientation::Direction;

use crate::geometry::CoordPair;

pub use thedes_entity::compact::InvalidId;

pub type Registry = thedes_entity::compact::Registry<Monster>;

pub type Id = thedes_entity::compact::ShortId;

pub type IdShortageError = thedes_entity::compact::NonShortId;

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
pub struct MonsterPosition {
    body: CoordPair,
    facing: Direction,
}

impl MonsterPosition {
    pub fn new(body: CoordPair, facing: Direction) -> Self {
        Self { body, facing }
    }

    pub fn body(&self) -> CoordPair {
        self.body
    }

    pub fn facing(&self) -> Direction {
        self.facing
    }

    pub(crate) fn set_body(&mut self, body: CoordPair) {
        self.body = body;
    }

    pub(crate) fn face(&mut self, facing: Direction) {
        self.facing = facing;
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Monster {
    position: MonsterPosition,
}

impl Monster {
    pub fn new(position: MonsterPosition) -> Self {
        Self { position }
    }

    pub fn position(&self) -> MonsterPosition {
        self.position
    }

    pub(crate) fn position_mut(&mut self) -> &mut MonsterPosition {
        &mut self.position
    }
}
