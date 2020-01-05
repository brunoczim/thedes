use ahash::AHasher;
use rand::{Rng, SeedableRng};
use std::hash::{Hash, Hasher};

/// A seed used for reproducible pseudo-random number generation.
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
pub struct Seed {
    bits: u64,
}

impl Seed {
    /// Generates a random seed.
    pub fn random() -> Self {
        Self { bits: rand::thread_rng().gen() }
    }

    /// Builds a seed over a given unsigned integer.
    pub fn from_u64(bits: u64) -> Self {
        Self { bits }
    }

    pub fn bits(self) -> u64 {
        self.bits
    }

    /// Builds a random number generator that will generate values associated
    /// with the given index object.
    pub fn make_rng<T>(self, index: T) -> impl Rng
    where
        T: Hash,
    {
        let mut hasher = AHasher::new_with_keys(0, 0);
        index.hash(&mut hasher);
        rand::rngs::StdRng::seed_from_u64(self.bits ^ hasher.finish())
    }
}
