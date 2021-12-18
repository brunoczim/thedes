use crate::map::Coord;
use gardiz::{coord::Vec2, direc::Direction};

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
pub struct Body {
    pub head: Vec2<Coord>,
    pub facing: Direction,
}

impl Body {
    #[inline]
    pub fn pointer(&self) -> Vec2<Coord> {
        self.head.move_one(self.facing)
    }
}
