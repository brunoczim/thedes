use crate::{
    map::Coord,
    math::rand::{
        noise::{NoiseGen, NoiseProcessor},
        weighted,
        Seed,
    },
    matter::ground::Ground,
};
use gardiz::coord::Vec2;
use rand::rngs::StdRng;
use std::fmt;

const SEED_SALT: u64 = 0x1E32CBEB51225355;

type Weight = u64;

const LOW_WEIGHT: Weight = 4;
const MID_WEIGHT: Weight = 5;
const HIGH_WEIGHT: Weight = 6;

const WEIGHTS: &[weighted::Entry<Biome, Weight>] = &[
    weighted::Entry { data: Biome::RockDesert, weight: LOW_WEIGHT },
    weighted::Entry { data: Biome::Plain, weight: MID_WEIGHT },
    weighted::Entry { data: Biome::Desert, weight: LOW_WEIGHT },
    weighted::Entry { data: Biome::Plain, weight: LOW_WEIGHT },
    weighted::Entry { data: Biome::RockDesert, weight: HIGH_WEIGHT },
    weighted::Entry { data: Biome::Desert, weight: MID_WEIGHT },
    weighted::Entry { data: Biome::RockDesert, weight: LOW_WEIGHT },
    weighted::Entry { data: Biome::Plain, weight: HIGH_WEIGHT },
    weighted::Entry { data: Biome::RockDesert, weight: MID_WEIGHT },
    weighted::Entry { data: Biome::Desert, weight: LOW_WEIGHT },
    weighted::Entry { data: Biome::Plain, weight: MID_WEIGHT },
    weighted::Entry { data: Biome::Desert, weight: HIGH_WEIGHT },
    weighted::Entry { data: Biome::RockDesert, weight: LOW_WEIGHT },
    weighted::Entry { data: Biome::Desert, weight: LOW_WEIGHT },
    weighted::Entry { data: Biome::Plain, weight: LOW_WEIGHT },
];

/// A biome type.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
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

/// A read-only map of biome types.
#[derive(Debug, Clone)]
pub struct Map {
    noise_gen: NoiseGen,
    noise_proc: weighted::Entries<Biome, Weight>,
}

impl Map {
    /// Creates a new map given a seed to create the noise function.
    pub fn new(seed: Seed) -> Self {
        let mut noise_gen = seed.make_noise_gen::<_, StdRng>(SEED_SALT);
        noise_gen.sensitivity = 0.0003;
        let noise_proc = weighted::Entries::new(WEIGHTS.iter().cloned());
        Self { noise_gen, noise_proc }
    }

    /// Gets a biome type at a given point.
    pub fn get(&self, point: Vec2<Coord>) -> Biome {
        (&&self.noise_proc).process(point, &self.noise_gen).data
    }
}

/// A weighted generator of biomes.
#[derive(Debug, Clone)]
pub struct Generator {
    noise_gen: NoiseGen,
    processor: weighted::Entries<Biome, Weight>,
}

impl Generator {
    /// Creates a new generator.
    pub fn new(seed: Seed) -> Generator {
        let mut noise_gen = seed.make_noise_gen::<_, StdRng>(SEED_SALT);
        noise_gen.sensitivity = 0.0003;
        let processor = weighted::Entries::new(WEIGHTS.iter().cloned());
        Self { noise_gen, processor }
    }

    /// Generates a biome tag at a given location.
    pub fn biome_at(&self, point: Vec2<Coord>) -> Biome {
        (&&self.processor).process(point, &self.noise_gen).data
    }
}
