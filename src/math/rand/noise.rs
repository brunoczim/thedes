use crate::math::{
    plane::{Coord2, Nat},
    rand::Seed,
};
use noise::{NoiseFn, Seedable};
use rand::{Rng, SeedableRng};
use std::hash::Hash;

/// The default noise generator.
#[derive(Debug, Clone)]
pub struct NoiseGen {
    inner: noise::Fbm,
    /// Sensitivity of this noise.
    pub sensitivity: f64,
}

impl NoiseGen {
    /// Builds noise generator that will generate values associated with the
    /// given index object.
    pub fn new<T, R>(seed: Seed, salt: T) -> NoiseGen
    where
        T: Hash,
        R: SeedableRng + Rng,
    {
        let seed = seed.make_rng::<T, R>(salt).gen();
        NoiseGen { inner: noise::Fbm::new().set_seed(seed), sensitivity: 1.0 }
    }

    #[inline]
    fn make_param(&self, param: f64) -> f64 {
        param * self.sensitivity
    }

    /// Generates noise from a slice of coordinates.
    pub fn gen_from_slice(&self, mut slice: &[f64]) -> f64 {
        let mut computed = None;

        let val = loop {
            match (slice.len(), computed) {
                (0, None) => break 0.0,
                (0, Some(val)) => break val,

                (1, None) => {
                    break self.inner.get([self.make_param(slice[0]), 0.0])
                },
                (1, Some(val)) => {
                    break self.inner.get([self.make_param(slice[0]), val])
                },

                (2, None) => {
                    break self.inner.get([
                        self.make_param(slice[0]),
                        self.make_param(slice[1]),
                    ])
                },
                (2, Some(val)) => {
                    break self.inner.get([
                        self.make_param(slice[0]),
                        self.make_param(slice[1]),
                        val,
                    ])
                },

                (3, None) => {
                    break self.inner.get([
                        self.make_param(slice[0]),
                        self.make_param(slice[1]),
                        self.make_param(slice[2]),
                    ])
                },
                (3, Some(val)) => {
                    break self.inner.get([
                        self.make_param(slice[0]),
                        self.make_param(slice[1]),
                        self.make_param(slice[2]),
                        val,
                    ])
                },

                (4, None) => {
                    break self.inner.get([
                        self.make_param(slice[0]),
                        self.make_param(slice[1]),
                        self.make_param(slice[2]),
                        self.make_param(slice[3]),
                    ])
                },

                (_, Some(val)) => {
                    computed = Some(self.inner.get([
                        self.make_param(slice[0]),
                        self.make_param(slice[1]),
                        self.make_param(slice[2]),
                        val,
                    ]));
                    slice = &slice[3 ..];
                },
                (_, None) => {
                    computed = Some(self.inner.get([
                        self.make_param(slice[0]),
                        self.make_param(slice[1]),
                        self.make_param(slice[2]),
                        self.make_param(slice[3]),
                    ]));
                    slice = &slice[4 ..];
                },
            }
        };

        (val + 1.0) / 2.0
    }

    /// Generates a random float based on the given input.
    pub fn gen<I>(&self, input: I) -> f64
    where
        I: NoiseInput,
    {
        input.apply_to(self)
    }
}

/// Data which can be applied to noise functions.
pub trait NoiseInput {
    /// Applies input to the given noise generator.
    fn apply_to(&self, gen: &NoiseGen) -> f64;
}

impl NoiseInput for Nat {
    fn apply_to(&self, gen: &NoiseGen) -> f64 {
        gen.gen_from_slice(&[*self as f64 + 0.5])
    }
}

impl NoiseInput for Coord2<Nat> {
    fn apply_to(&self, gen: &NoiseGen) -> f64 {
        gen.gen_from_slice(&[self.x as f64 + 0.5, self.y as f64 + 0.5])
    }
}

impl<'input, T> NoiseInput for &'input T
where
    T: NoiseInput + ?Sized,
{
    fn apply_to(&self, gen: &NoiseGen) -> f64 {
        (**self).apply_to(gen)
    }
}

/// A type that processes noise into a concrete value.
pub trait NoiseProcessor<I, T>
where
    I: NoiseInput,
{
    /// Calls the noise generator as much as need to build the output value.
    fn process(&self, input: I, gen: &NoiseGen) -> T;
}
