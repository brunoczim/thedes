pub mod noise;

use ahash::AHasher;
use num::traits::{cast::ToPrimitive, NumAssign};
use rand::{thread_rng, Rng, SeedableRng};
use std::hash::{Hash, Hasher};
use thedes_common::seed::Seed;

pub fn random_seed() -> Seed {
    Seed { bits: thread_rng().gen() }
}

pub fn make_rng<T, R>(seed: Seed, salt: T) -> R
where
    T: Hash,
    R: SeedableRng,
{
    let mut hasher = AHasher::new_with_keys(0, 0);
    salt.hash(&mut hasher);
    R::seed_from_u64(seed.bits ^ hasher.finish())
}

/// Step of range.
pub trait Step: Ord + NumAssign + Clone + ToPrimitive {}

impl<T> Step for T where T: Ord + NumAssign + Clone + ToPrimitive {}

/// Iterator over any number-typed range.
#[derive(Debug, Clone)]
pub struct NumRange<T>
where
    T: Step,
{
    /// Inclusive start,
    pub start: T,
    /// Inclusive start,
    pub end: T,
}

impl<T> Iterator for NumRange<T>
where
    T: Step,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let curr = self.start.clone();
            self.start += T::one();
            Some(curr)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = if self.end < self.start {
            0
        } else {
            (self.end.clone() - self.start.clone())
                .to_usize()
                .expect("range is out of bounds")
        };
        (hint, Some(hint))
    }
}

impl<T> DoubleEndedIterator for NumRange<T>
where
    T: Step,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            self.end -= T::one();
            Some(self.end.clone())
        } else {
            None
        }
    }
}

impl<T> ExactSizeIterator for NumRange<T> where T: Step {}
