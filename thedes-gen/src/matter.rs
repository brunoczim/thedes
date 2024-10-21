use std::array;

use rand::Rng;
use rand_distr::Distribution;
use thedes_domain::{bitpack::BitPack, matter::Biome};

use super::random::ProabilityWeight;

#[derive(Debug, Clone)]
pub struct BiomeDist {
    cumulative_weights: [ProabilityWeight; Biome::ELEM_COUNT],
}

impl Default for BiomeDist {
    fn default() -> Self {
        Self::new(|ground| match ground {
            Biome::Plains => 11,
            Biome::Desert => 5,
            Biome::Wasteland => 4,
        })
    }
}

impl BiomeDist {
    pub fn new<F>(mut density_function: F) -> Self
    where
        F: FnMut(Biome) -> ProabilityWeight,
    {
        let mut accumuled_weight = 0;
        let cumulative_weights = array::from_fn(|i| {
            accumuled_weight += density_function(Biome::ALL[i]);
            accumuled_weight
        });
        Self { cumulative_weights }
    }
}

impl Distribution<Biome> for BiomeDist {
    fn sample<R>(&self, rng: &mut R) -> Biome
    where
        R: Rng + ?Sized,
    {
        let last_cumulative_weight =
            self.cumulative_weights[self.cumulative_weights.len() - 1];
        let sampled_weight = rng.gen_range(0 .. last_cumulative_weight);
        for (i, cumulative_weight) in
            self.cumulative_weights.into_iter().enumerate()
        {
            if sampled_weight < cumulative_weight {
                return Biome::ALL[i];
            }
        }
        panic!("sampled weight {sampled_weight} is out of requested bounds")
    }
}
