use crate::{
    math::{
        plane::{Coord2, Nat},
        rand::{
            noise::{NoiseGen, NoiseInput, NoiseProcessor},
            weight::{Weighted, WeightedNoise},
            Seed,
        },
    },
    matter::ground::Ground,
};
use rand::rngs::StdRng;
use std::fmt;

const SEED_SALT: u64 = 0x1E32CBEB51225355;

const LOW_WEIGHT: u64 = 4;
const MID_WEIGHT: u64 = 5;
const HIGH_WEIGHT: u64 = 6;

const WEIGHTS: &'static [Weighted<Biome>] = &[
    Weighted { data: Biome::RockDesert, weight: LOW_WEIGHT },
    Weighted { data: Biome::Plain, weight: MID_WEIGHT },
    Weighted { data: Biome::Desert, weight: LOW_WEIGHT },
    Weighted { data: Biome::Plain, weight: LOW_WEIGHT },
    Weighted { data: Biome::RockDesert, weight: HIGH_WEIGHT },
    Weighted { data: Biome::Desert, weight: MID_WEIGHT },
    Weighted { data: Biome::RockDesert, weight: LOW_WEIGHT },
    Weighted { data: Biome::Plain, weight: HIGH_WEIGHT },
    Weighted { data: Biome::RockDesert, weight: MID_WEIGHT },
    Weighted { data: Biome::Desert, weight: LOW_WEIGHT },
    Weighted { data: Biome::Plain, weight: MID_WEIGHT },
    Weighted { data: Biome::Desert, weight: HIGH_WEIGHT },
    Weighted { data: Biome::RockDesert, weight: LOW_WEIGHT },
    Weighted { data: Biome::Desert, weight: LOW_WEIGHT },
    Weighted { data: Biome::Plain, weight: LOW_WEIGHT },
];

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

/// A type that computes biomes from noise.
#[derive(Debug, Clone)]
pub struct FromNoise {
    weighted: WeightedNoise,
}

impl FromNoise {
    /// Initializes this processor.
    pub fn new() -> Self {
        let weighted =
            WeightedNoise::new(WEIGHTS.iter().map(|pair| pair.weight));
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
        WEIGHTS[index].data.clone()
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
                noise.sensitivity = 0.0003;
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
