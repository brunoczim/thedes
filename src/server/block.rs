use crate::{
    common::block::Block,
    map::Coord,
    math::rand::{weighted, Seed},
};
use gardiz::coord::Vec2;
use rand::{rngs::StdRng, Rng};

const SEED_SALT: u64 = 0x10253E6093C603D;

type Weight = u64;

const WEIGHTS: &[weighted::Entry<Block, Weight>] = &[
    weighted::Entry { data: Block::Empty, weight: 100 * 100 },
    weighted::Entry { data: Block::Twig, weight: 1 },
];

/// A weighted generator of blocks.
#[derive(Debug, Clone)]
pub struct Generator {
    seed: Seed,
    weights: weighted::Entries<Block, Weight>,
}

/*
impl Generator {
    /// Creates a new generator.
    pub fn new(seed: Seed) -> Generator {
        let weights = weighted::Entries::new(WEIGHTS.iter().cloned());
        Self { seed, weights }
    }

    pub fn block_at(&self, point: Vec2<Coord>) -> Block {
        self.seed
            .make_rng::<_, StdRng>((SEED_SALT, point))
            .sample(&self.weights)
            .data
            .clone()
    }
}
*/
