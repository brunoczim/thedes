use thedes_domain::thede::{AllocError, Id, Registry};
use thiserror::Error;

use crate::random::{MutableDistribution, ProabilityWeight};

#[derive(Debug, Error)]
pub enum ThedeDistrError {
    #[error("Failed to allocate a new Thede ID")]
    AllocError(
        #[source]
        #[from]
        AllocError,
    ),
}

#[derive(Debug, Clone)]
pub struct ThedeDistrConfig {
    new_thede_weight: ProabilityWeight,
    unclaimed_weight: ProabilityWeight,
}

impl Default for ThedeDistrConfig {
    fn default() -> Self {
        Self { new_thede_weight: 1, unclaimed_weight: 11 }
    }
}

impl ThedeDistrConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_new_thede_weight(self, weight: ProabilityWeight) -> Self {
        Self { new_thede_weight: weight, ..self }
    }

    pub fn with_unclaimed_weight(self, weight: ProabilityWeight) -> Self {
        Self { unclaimed_weight: weight, ..self }
    }

    pub fn finish(self) -> ThedeDistr {
        ThedeDistr { registry: Registry::all_free(), config: self }
    }
}

#[derive(Debug, Clone)]
pub struct ThedeDistr {
    registry: Registry,
    config: ThedeDistrConfig,
}

impl Default for ThedeDistr {
    fn default() -> Self {
        ThedeDistrConfig::new().finish()
    }
}

impl ThedeDistr {
    pub fn finish(self) -> Registry {
        self.registry
    }

    fn cumulative_weight(&self) -> ProabilityWeight {
        self.config.new_thede_weight + self.config.unclaimed_weight
    }
}

impl MutableDistribution<Option<Id>> for ThedeDistr {
    type Error = ThedeDistrError;

    fn sample_mut<R>(&mut self, rng: &mut R) -> Result<Option<Id>, Self::Error>
    where
        R: rand::Rng + ?Sized,
    {
        let total = rng.gen_range(0 .. self.cumulative_weight());
        if total < self.config.new_thede_weight {
            let id = self.registry.alloc()?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }
}
