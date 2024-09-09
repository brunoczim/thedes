use thedes_geometry::axis::Direction;
use thiserror::Error;

use crate::{geometry::CoordPair, item::Inventory};

#[derive(Debug, Error)]
pub enum CreationError {
    #[error("Player pointer position would overflow")]
    Overflow,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerPosition {
    head: CoordPair,
    facing: Direction,
}

impl PlayerPosition {
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
    inventory: Inventory,
}

impl Player {
    pub fn new(position: PlayerPosition, inventory: Inventory) -> Self {
        Self { position, inventory }
    }

    pub fn position(&self) -> &PlayerPosition {
        &self.position
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub(crate) fn position_mut(&mut self) -> &mut PlayerPosition {
        &mut self.position
    }

    pub(crate) fn inventory_mut(&mut self) -> &mut Inventory {
        &mut self.inventory
    }
}
