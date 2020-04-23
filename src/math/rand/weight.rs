use crate::{
    error::Result,
    math::rand::noise::{NoiseGen, NoiseInput, NoiseProcessor},
};
use num::traits::{NumAssign, ToPrimitive};
use std::{error::Error, fmt};

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

impl Error for WeightError {}

/// A weighted noise processor.
#[derive(Debug, Clone)]
pub struct WeightedNoise {
    sums: Vec<Weight>,
}

impl WeightedNoise {
    /// Builds a new weighted noise processor.
    ///
    /// # Panics
    /// Panics if weights overflow.
    pub fn new<I>(weights: I) -> Self
    where
        I: IntoIterator<Item = Weight>,
    {
        Self::try_new(weights).expect("WeightedNoise::new failed")
    }

    /// Builds a new weighted noise processor. Returns error if weights
    /// overflow.
    pub fn try_new<I>(weights: I) -> Result<Self>
    where
        I: IntoIterator<Item = Weight>,
    {
        let mut sums: Vec<Weight> = weights.into_iter().collect();

        if sums.len() == 0 {
            Err(WeightError::Empty)?;
        }

        let mut acc: Weight = 0;

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
        let search = scaled as Weight + 1;
        self.sums
            .binary_search(&search)
            .unwrap_or_else(|index| index.min(self.sums.len() - 1))
    }
}

/// Integer that represents a weight.
pub type Weight = u64;

/// Data that is weighted.
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
pub struct Weighted<T> {
    /// The weight of the data.
    pub weight: Weight,
    /// The data to which the weight is associated to.
    pub data: T,
}

impl<T> Default for Weighted<T>
where
    T: Default,
{
    fn default() -> Self {
        Self { weight: 1, data: T::default() }
    }
}

/// Iterator over a absolute value function of weights.
#[derive(Debug)]
pub struct TentWeightFn<I>
where
    I: Iterator,
{
    iter: I,
    low: Weight,
    high: Weight,
    peak: usize,
    size: usize,
    curr: usize,
}

impl<I> TentWeightFn<I>
where
    I: Iterator,
{
    /// Creates an iterator over linear growth and linear decrease of weights,
    /// in a fashion of tent map functions.
    ///
    /// The range of weights is `[low, high)` (high excluded). Peak is the index
    /// of the peak of the weight function
    ///
    /// # Panics
    /// Panics if `low >= high`.
    pub fn new<J>(iter: J, low: Weight, high: Weight, peak: usize) -> Self
    where
        J: IntoIterator<IntoIter = I, Item = I::Item>,
        I: ExactSizeIterator,
    {
        if low >= high {
            panic!("Cannot have low >= high in tent fn weights");
        }

        let iter = iter.into_iter();
        TentWeightFn { size: iter.len(), iter, low, high, peak, curr: 0 }
    }
}

impl<T> TentWeightFn<NumRange<T>>
where
    T: NumAssign + PartialOrd + Clone + ToPrimitive,
{
    /// Creates an iterator over linear growth and linear decrease of weights,
    /// in a fashion of tent map functions.
    ///
    /// The range of iteration is [start, end) (high excluded). Peak is the
    /// index of the peak of the weight function. Weight bounds are `[1,
    /// (end - start + 1) / 2))`.
    ///
    /// # Panics
    /// Panics if `start >= end`.
    pub fn from_range(start: T, end: T) -> Self {
        if start >= end {
            panic!("Cannot have start >= end in tent fn weights");
        }
        let range = NumRange { start, end };
        let len = range.len();
        let half = (len + 1) / 2;
        Self::new(range, 1, half as Weight + 1, half)
    }
}

impl<I> Iterator for TentWeightFn<I>
where
    I: Iterator,
{
    type Item = Weighted<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.iter.next()?;
        let curr = self.curr as f64;
        let size = self.size as f64;
        let range = (self.high - self.low) as f64;
        let offset = (curr * range / size) as Weight;
        let weight = self.low + offset;
        self.curr += 1;
        Some(Weighted { data, weight })
    }
}

/// Iterator over any number-typed range.
#[derive(Debug, Clone)]
pub struct NumRange<T>
where
    T: NumAssign + PartialOrd + Clone + ToPrimitive,
{
    /// Inclusive start,
    pub start: T,
    /// Inclusive start,
    pub end: T,
}

impl<T> Iterator for NumRange<T>
where
    T: NumAssign + PartialOrd + Clone + ToPrimitive,
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

impl<T> ExactSizeIterator for NumRange<T> where
    T: NumAssign + PartialOrd + Clone + ToPrimitive
{
}
