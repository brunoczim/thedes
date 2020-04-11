use crate::rand::{NoiseGen, NoiseInput, NoiseProcessor, WeightedNoise};

const WEIGHTS: &'static [(Ground, u64)] = &[
    (Ground::Grass, 1),
    (Ground::Sand, 2),
    (Ground::Grass, 1),
    (Ground::Sand, 2),
    (Ground::Grass, 2),
    (Ground::Sand, 1),
    (Ground::Grass, 2),
    (Ground::Sand, 1),
];

/// A ground block (background color).
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
pub enum Ground {
    /// This block's ground is grass.
    Grass,
    /// This block's ground is sand.
    Sand,
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
    type Output = Ground;

    fn process(&self, input: I, gen: &NoiseGen) -> Self::Output {
        let index = self.weighted.process(input, gen);
        let (ground, _) = &WEIGHTS[index];
        ground.clone()
    }
}
