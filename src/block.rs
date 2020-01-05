use crate::entity;
use rand::{distributions::Distribution, Rng};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// A single block in the game.
pub enum Block {
    /// Empty.
    Empty,
    /// Entity is occupying this block.
    Entity(entity::Id),
}

#[derive(Debug)]
pub struct BlockDist;

impl Distribution<Block> for BlockDist {
    fn sample<R>(&self, _rng: &mut R) -> Block
    where
        R: Rng + ?Sized,
    {
        Block::Empty
    }
}
