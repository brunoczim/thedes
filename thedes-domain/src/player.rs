use thedes_geometry::axis::Direction;
use thiserror::Error;

use crate::geometry::CoordPair;

#[derive(Debug, Error)]
pub enum CreationError {
    #[error("Player pointer position would overflow")]
    Overflow,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Player {
    head: CoordPair,
    facing: Direction,
}

impl Player {
    pub fn new(
        head: CoordPair,
        facing: Direction,
    ) -> Result<Self, CreationError> {
        if head.checked_move_unit(facing).is_none() {
            Err(CreationError::Overflow)?
        }
        Ok(Self { head, facing })
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
