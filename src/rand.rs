use crate::{
    coord::{Coord2, Nat},
    error::{Error, Result},
};
use ahash::AHasher;
use noise::{NoiseFn, Seedable};
use rand::{Rng, SeedableRng};
use std::{
    error::Error as StdError,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
/// Error returned by [`weight::try_new`].
pub enum WeightError {
    /// Happens when an overflow occurs computing the weights' sum.
    Overflow,
    /// Happens when an empty array of weights is given.
    Empty,
    /// Happens when all weights are zeroes.
    Zeroes,
}

impl fmt::Display for WeightError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(match self {
            Self::Overflow => {
                "weighted noise processor overflowed on the weights"
            },
            Self::Empty => "Weight array is empty",
            Self::Zeroes => "all weights are zeroes",
        })
    }
}

impl StdError for WeightError {}

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
        let seed = self.make_rng::<T, R>(salt).gen();
        NoiseGen {
            inner: noise::Perlin::new().set_seed(seed),
            sensitivity: 1.0,
        }
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

/// The default noise generator.
#[derive(Debug, Clone)]
pub struct NoiseGen {
    inner: noise::Perlin,
    /// Sensitivity of this noise.
    pub sensitivity: f64,
}

impl NoiseGen {
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
pub trait NoiseProcessor<I>
where
    I: NoiseInput,
{
    /// The value generated by this noise processor.
    type Output;

    /// Calls the noise generator as much as need to build the output value.
    fn process(&self, input: I, gen: &NoiseGen) -> Self::Output;
}

/// A weighted noise processor.
#[derive(Debug, Clone)]
pub struct WeightedNoise {
    sums: Vec<u64>,
}

impl WeightedNoise {
    /// Builds a new weighted noise processor.
    ///
    /// # Panics
    /// Panics if weights overflow.
    pub fn new<I>(weights: I) -> Self
    where
        I: IntoIterator<Item = u64>,
    {
        Self::try_new(weights).expect("WeightedNoise::new failed")
    }

    /// Builds a new weighted noise processor. Returns error if weights
    /// overflow.
    pub fn try_new<I>(weights: I) -> Result<Self>
    where
        I: IntoIterator<Item = u64>,
    {
        let mut sums: Vec<u64> = weights.into_iter().collect();

        if sums.len() == 0 {
            Err(WeightError::Empty)?;
        }

        let mut acc = 0u64;

        for elem in &mut sums {
            acc = acc.checked_add(*elem).ok_or(WeightError::Overflow)?;
            *elem = acc
        }

        if acc == 0 {
            Err(WeightError::Zeroes)?;
        }

        Ok(Self { sums })
    }
}

impl<I> NoiseProcessor<I> for WeightedNoise
where
    I: NoiseInput,
{
    type Output = usize;

    fn process(&self, input: I, generator: &NoiseGen) -> Self::Output {
        let noise = generator.gen(input);
        let scale = *self.sums.last().expect("checked on new");
        // loss ahead
        let scaled = noise * scale as f64;
        let search = scaled as u64 + 1;
        self.sums
            .binary_search(&search)
            .unwrap_or_else(|index| index.min(self.sums.len() - 1))
    }
}
