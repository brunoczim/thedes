use crate::rand::{NoiseGen, NoiseInput, NoiseProcessor, WeightedNoise};

const WEIGHTS: &'static [(Block, u64)] = &[(Block::Empty, 3), (Block::Wall, 2)];

/// Kind of a block.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Block {
    /// Empty.
    Empty,
    /// Wall block.
    Wall,
}

/// A type that computes blocks from noise.
#[derive(Debug, Clone)]
pub struct FromNoise {
    weighted: WeightedNoise,
}

impl FromNoise {
    /// Initializes this processor.
    pub fn new() -> Self {
        let weighted =
            WeightedNoise::new(WEIGHTS.iter().map(|&(_, weight)| weight));
        Self { weighted }
    }
}

impl<I> NoiseProcessor<I> for FromNoise
where
    I: NoiseInput,
{
    type Output = Block;

    fn process(&self, input: I, gen: &NoiseGen) -> Self::Output {
        let index = self.weighted.process(input, gen);
        let (block, _) = &WEIGHTS[index];
        block.clone()
    }
}
