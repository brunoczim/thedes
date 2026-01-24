use serde::{Deserialize, Serialize};
use thedes_geometry::orientation::Direction;
use thiserror::Error;

use crate::{
    geometry::CoordPair,
    stat::{Stat, StatValue},
};

#[derive(Debug, Error)]
pub enum InvalidPlayerHp {
    #[error("Player HP {0} is too big")]
    TooBig(u8),
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Player pointer position would overflow")]
    Overflow,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
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

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Player {
    position: PlayerPosition,
    hp: Stat,
}

impl Player {
    pub const DEFAULT_HP: Stat = Stat::new(80, 80);

    pub fn new(position: PlayerPosition, hp: Stat) -> Self {
        Self { position, hp }
    }

    pub fn position(&self) -> &PlayerPosition {
        &self.position
    }

    pub(crate) fn position_mut(&mut self) -> &mut PlayerPosition {
        &mut self.position
    }

    pub fn hp(&self) -> Stat {
        self.hp
    }

    pub fn damage(&mut self, amount: StatValue) {
        self.hp.decrease_value(amount);
    }

    pub fn heal(&mut self, amount: StatValue) {
        self.hp.increase_value(amount);
    }
}
