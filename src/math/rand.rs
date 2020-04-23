/// Utilities related to weightening number generators.
pub mod weight;

/// Utilities related to noise number generators.
pub mod noise;

use self::noise::NoiseGen;
use crate::error::{Error, Result};
use ahash::AHasher;
use rand::{thread_rng, Rng, SeedableRng};
use std::{
    error::Error as StdError,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

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
        Self { bits: thread_rng().gen() }
    }

    /// Builds a seed over a given unsigned integer.
    pub fn from_u64(bits: u64) -> Self {
        Self { bits }
    }

    /// Bits of the seed.
    pub fn bits(self) -> u64 {
        self.bits
    }

    /// Builds a random number generator that will generate values associated
    /// with the given index object.
    pub fn make_rng<T, R>(self, salt: T) -> R
    where
        T: Hash,
        R: SeedableRng,
    {
        let mut hasher = AHasher::new_with_keys(0, 0);
        salt.hash(&mut hasher);
        R::seed_from_u64(self.bits ^ hasher.finish())
    }

    /// Builds noise generator that will generate values associated with the
    /// given index object.
    pub fn make_noise_gen<T, R>(self, salt: T) -> NoiseGen
    where
        T: Hash,
        R: SeedableRng + Rng,
    {
        NoiseGen::new::<T, R>(self, salt)
    }
}

/// Error generated when a non-hex number is given as `Seed::from_str` input.
#[derive(Debug, Clone, Default)]
pub struct NotHex;

impl fmt::Display for NotHex {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad("Input is not a 16-digit hexadecimal number")
    }
}

impl StdError for NotHex {}

impl FromStr for Seed {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        let string = string.trim();
        if string.len() == 0 || string.len() > 16 {
            Err(NotHex)?;
        }

        let mut bits = 0;

        for ch in string.chars() {
            bits *= 16;
            if ch.is_ascii_lowercase() {
                bits += (10 + ch as u32 - 'a' as u32) as u64;
            } else if ch.is_ascii_uppercase() {
                bits += (10 + ch as u32 - 'A' as u32) as u64;
            } else if ch.is_ascii_digit() {
                bits += (ch as u32 - '0' as u32) as u64;
            } else {
                Err(NotHex)?;
            }
        }

        Ok(Seed { bits })
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let buf = format!("{:x}", self.bits());
        fmt.pad(&buf)
    }
}
