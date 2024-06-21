use thedes_geometry::axis::Direction;

use crate::geometry::CoordPair;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Player {
    head: CoordPair,
    facing: Direction,
}

impl Player {
    pub(crate) fn new(head: CoordPair, facing: Direction) -> Self {
        Self { head, facing }
    }

    pub fn head(&self) -> CoordPair {
        self.head
    }

    pub fn set_head(&mut self, new_head: CoordPair) {
        self.head = new_head;
    }

    pub fn facing(&self) -> Direction {
        self.facing
    }

    pub(crate) fn face(&mut self, direction: Direction) {
        self.facing = direction;
    }

    pub fn pointer(&self) -> CoordPair {
        self.head.move_unit(self.facing)
    }
}
