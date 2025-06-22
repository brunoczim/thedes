use std::convert::Infallible;

use rand::{Rng, SeedableRng};
use rand_distr::Distribution;

pub type PickedReproducibleRng = rand_chacha::ChaCha8Rng;

pub type ProabilityWeight = u32;

pub type Seed = u32;

pub fn create_reproducible_rng(seed: Seed) -> PickedReproducibleRng {
    let mut full_seed = <PickedReproducibleRng as SeedableRng>::Seed::default();
    for (i, chunk) in full_seed.chunks_exact_mut(4).enumerate() {
        let i = i as Seed;
        let bits = seed.wrapping_sub(i) ^ (i << 14);
        chunk.copy_from_slice(&bits.to_le_bytes());
    }
    PickedReproducibleRng::from_seed(full_seed)
}

pub trait MutableDistribution<T> {
    type Error: std::error::Error;

    fn sample_mut<R>(&mut self, rng: &mut R) -> Result<T, Self::Error>
    where
        R: Rng + ?Sized;
}

impl<T, D> MutableDistribution<T> for D
where
    D: Distribution<T>,
{
    type Error = Infallible;

    fn sample_mut<R>(&mut self, rng: &mut R) -> Result<T, Self::Error>
    where
        R: Rng + ?Sized,
    {
        Ok(self.sample(rng))
    }
}
