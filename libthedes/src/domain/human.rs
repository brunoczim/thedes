use gardiz::{coord::Vec2, direc::Direction};

use super::plane::Coord;

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
pub struct Location {
    pub head: Vec2<Coord>,
    pub facing: Direction,
}

impl Location {
    pub fn pointer(self) -> Vec2<Coord> {
        self.head.move_one(self.facing)
    }

    pub fn checked_pointer(self) -> Option<Vec2<Coord>> {
        self.head.checked_move(self.facing)
    }
}
