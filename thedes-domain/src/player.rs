use thedes_geometry::orientation::Direction;
use thiserror::Error;

use crate::geometry::CoordPair;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Player pointer position would overflow")]
    Overflow,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerPosition {
    head: CoordPair,
    facing: Direction,
}

impl PlayerPosition {
    pub fn new(head: CoordPair, facing: Direction) -> Result<Self, InitError> {
        if head.checked_move_unit(facing).is_none() {
            Err(InitError::Overflow)?
        }
        Ok(Self { head, facing })
    }

    pub fn head(&self) -> CoordPair {
        self.head
    }

    pub(crate) fn set_head(&mut self, new_head: CoordPair) {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Player {
    position: PlayerPosition,
}

impl Player {
    pub fn new(position: PlayerPosition) -> Self {
        Self { position }
    }

    pub fn position(&self) -> &PlayerPosition {
        &self.position
    }

    pub(crate) fn position_mut(&mut self) -> &mut PlayerPosition {
        &mut self.position
    }
}
