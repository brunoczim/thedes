use std::array;

use rand::Rng;
use rand_distr::Distribution;
use thedes_domain::matter::Ground;

use super::random::ProabilityWeight;

#[derive(Debug, Clone)]
pub struct GroundDist {
    cumulative_weights: [ProabilityWeight; 3],
}

impl Default for GroundDist {
    fn default() -> Self {
        Self::new(|ground| match ground {
            Ground::Grass => 11,
            Ground::Sand => 5,
            Ground::Stone => 4,
        })
    }
}

impl GroundDist {
    pub fn new<F>(mut density_function: F) -> Self
    where
        F: FnMut(Ground) -> ProabilityWeight,
    {
        let mut accumuled_weight = 0;
        let cumulative_weights = array::from_fn(|i| {
            accumuled_weight += density_function(Ground::ALL[i]);
            accumuled_weight
        });
        Self { cumulative_weights }
    }
}

impl Distribution<Ground> for GroundDist {
    fn sample<R>(&self, rng: &mut R) -> Ground
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
                return Ground::ALL[i];
            }
        }
        panic!("sampled weight {sampled_weight} is out of requested bounds")
    }
}
