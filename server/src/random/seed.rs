use self::noise::NoiseGen;
use crate::error::{Error, Result};
use ahash::AHasher;
use rand::{thread_rng, Rng, SeedableRng};
use std::{
    error::Error as StdError,
    fmt,
    hash::{Hash, Hasher},
};

pub fn make_noise_gen<T, R>(seed: Seed, salt: T) -> NoiseGen
where
    T: Hash,
    R: SeedableRng + Rng,
{
    NoiseGen::new::<T, R>(seed, salt)
}
