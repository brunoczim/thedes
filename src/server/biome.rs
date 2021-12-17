use crate::{
    common::biome::Biome,
    map::Coord,
    math::rand::{
        noise::{NoiseGen, NoiseProcessor},
        weighted,
        Seed,
    },
};
use gardiz::coord::Vec2;
use rand::rngs::StdRng;

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
