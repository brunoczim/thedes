use crate::{
    coord::{Coord2, Nat},
    matter::ground::Ground,
    rand::{NoiseGen, NoiseInput, NoiseProcessor, Seed, WeightedNoise},
};
use rand::rngs::StdRng;
use std::fmt;

/// A biome type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Biome {
    /// This biome is a plain.
    Plain,
    /// This biome is a sand desert.
    Desert,
    /// This biome is a rock desert.
    RockDesert,
}

impl fmt::Display for Biome {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(match self {
            Biome::Plain => "plain",
            Biome::Desert => "desert",
            Biome::RockDesert => "rocks",
        })
    }
}

impl Biome {
    /// Returns the main ground type of this biome.
    pub fn main_ground(&self) -> Ground {
        match self {
            Biome::Plain => Ground::Grass,
            Biome::Desert => Ground::Sand,
            Biome::RockDesert => Ground::Rock,
        }
    }
}

const SEED_SALT: u128 = 0xDC7A4811D0EA7CB11E32CBEB51225355;

const LOW_WEIGHT: u64 = 1;
const MID_WEIGHT: u64 = 2;
const HIGH_WEIGHT: u64 = 3;

const WEIGHTS: &'static [(Biome, u64)] = &[
    (Biome::RockDesert, LOW_WEIGHT),
    (Biome::Plain, MID_WEIGHT),
    (Biome::Desert, LOW_WEIGHT),
    (Biome::Plain, LOW_WEIGHT),
    (Biome::RockDesert, HIGH_WEIGHT),
    (Biome::Desert, MID_WEIGHT),
    (Biome::RockDesert, LOW_WEIGHT),
    (Biome::Plain, HIGH_WEIGHT),
    (Biome::RockDesert, MID_WEIGHT),
    (Biome::Desert, LOW_WEIGHT),
    (Biome::Plain, MID_WEIGHT),
    (Biome::Desert, HIGH_WEIGHT),
    (Biome::RockDesert, LOW_WEIGHT),
    (Biome::Desert, LOW_WEIGHT),
    (Biome::Plain, LOW_WEIGHT),
];

/// A type that computes biomes from noise.
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
    type Output = Biome;

    fn process(&self, input: I, gen: &NoiseGen) -> Self::Output {
        let index = self.weighted.process(input, gen);
        let (biome, _) = &WEIGHTS[index];
        biome.clone()
    }
}

/// A read-only map of biome types.
#[derive(Debug, Clone)]
pub struct Map {
    noise_gen: NoiseGen,
    noise_proc: FromNoise,
}

impl Map {
    /// Creates a new map given a seed to create the noise function.
    pub fn new(seed: Seed) -> Self {
        Self {
            noise_gen: {
                let mut noise = seed.make_noise_gen::<_, StdRng>(SEED_SALT);
                noise.sensitivity = 0.00075;
                noise
            },
            noise_proc: FromNoise::new(),
        }
    }

    /// Gets a biome type at a given point.
    pub fn get(&self, point: Coord2<Nat>) -> Biome {
        self.noise_proc.process(point, &self.noise_gen)
    }
}