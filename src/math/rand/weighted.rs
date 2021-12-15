use crate::{
    error::Result,
    math::{
        rand::noise::{NoiseGen, NoiseInput, NoiseProcessor},
        NumRange,
    },
};
use num::{
    integer::Integer,
    rational::Ratio,
    traits::{
        cast::{FromPrimitive, ToPrimitive},
        ops::checked::{
            CheckedAdd,
            CheckedDiv,
            CheckedMul,
            CheckedRem,
            CheckedSub,
        },
        NumAssign,
    },
};
use rand::{
    distributions::{uniform::SampleUniform, Distribution},
    Rng,
};
use std::{error::Error, fmt, ops::Rem};

/// Happens when an overflow occurs computing the weights' sum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Overflow;

impl fmt::Display for Overflow {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad("weighted noise processor overflowed on the weights")
    }
}

impl Error for Overflow {}

/// Happens when an empty array of weights is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct EmptyEntries;

impl fmt::Display for EmptyEntries {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad("Weight array is empty")
    }
}

impl Error for EmptyEntries {}

/// Happens when all weights are zeroes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct ZeroedEntries;

impl fmt::Display for ZeroedEntries {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad("all weights are zeroes")
    }
}

impl Error for ZeroedEntries {}

/// Weight trait bounds. In order to use noise processor, `Ratio::<W>::from_f64`
/// must be supported.
pub trait Weight
where
    Self: Integer
        + NumAssign
        + FromPrimitive
        + ToPrimitive
        + Rem
        + CheckedAdd
        + CheckedSub
        + CheckedMul
        + CheckedDiv
        + CheckedRem
        + Clone,
{
}

impl<W> Weight for W where
    W: Integer
        + NumAssign
        + FromPrimitive
        + ToPrimitive
        + Rem
        + CheckedAdd
        + CheckedSub
        + CheckedMul
        + CheckedDiv
        + CheckedRem
        + Clone
{
}

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
pub struct Entry<T, W>
where
    W: Weight,
{
    /// The data to which the weight is associated to.
    pub data: T,
    /// The weight of the data.
    pub weight: W,
}

impl<T, W> Default for Entry<T, W>
where
    T: Default,
    W: Weight,
    Ratio<W>: FromPrimitive,
{
    fn default() -> Self {
        Self { weight: W::one(), data: T::default() }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Indices<W> {
    sums: Vec<W>,
}

impl<W> Indices<W>
where
    W: Weight,
{
    /// Builds a new weighted random generator.
    ///
    /// # Panics
    /// Panics if weights overflow.
    pub fn new<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = W>,
    {
        Self::try_new(entries).expect("Entries::new failed")
    }

    /// Builds a new weighted random generator. Returns error if weights
    /// overflow or if sum of weights are zero.
    pub fn try_new<I>(entries: I) -> Result<Self>
    where
        I: IntoIterator<Item = W>,
    {
        let mut sums: Vec<W> = entries.into_iter().collect();

        let (mut head, tail) = sums.split_first_mut().ok_or(EmptyEntries)?;

        for elem in tail {
            *elem = elem.checked_add(&head).ok_or(Overflow)?;
            head = elem;
        }

        if head.is_zero() {
            Err(ZeroedEntries)?;
        }

        Ok(Self { sums })
    }
}

impl<I, W> NoiseProcessor<I, usize> for Indices<W>
where
    W: Weight,
    I: NoiseInput,
{
    fn process(&self, input: I, generator: &NoiseGen) -> usize {
        let noise = generator.gen(input);
        let scale = self
            .sums
            .last()
            .expect("checked on new")
            .to_f64()
            .expect("conversion to f64 is required");
        let scaled = noise * scale;
        let search =
            W::from_f64(scaled + 1.0).expect("conversion from f64 is required");
        self.sums
            .binary_search(&search)
            .unwrap_or_else(|index| index.min(self.sums.len() - 1))
    }
}

impl<W> Distribution<usize> for Indices<W>
where
    W: Weight + SampleUniform,
{
    fn sample<R>(&self, rng: &mut R) -> usize
    where
        R: Rng + ?Sized,
    {
        let last = self.sums.last().expect("checked on new").clone();
        let generated = rng.gen_range(W::zero(), last);
        let search = generated + W::one();
        self.sums
            .binary_search(&search)
            .unwrap_or_else(|index| index.min(self.sums.len() - 1))
    }
}

/// A weighted random generator that generates references to entries.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Entries<T, W>
where
    W: Weight,
{
    entries: Vec<Entry<T, W>>,
    indices: Indices<W>,
}

impl<T, W> Entries<T, W>
where
    W: Weight,
{
    /// Builds a new weighted random generator.
    ///
    /// # Panics
    /// Panics if weights overflow.
    pub fn new<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = Entry<T, W>>,
    {
        Self::try_new(entries).expect("Entries::new failed")
    }

    /// Builds a new weighted random generator. Returns error if weights
    /// overflow or if sum of weights are zero.
    pub fn try_new<I>(entries: I) -> Result<Self>
    where
        I: IntoIterator<Item = Entry<T, W>>,
    {
        let entries: Vec<Entry<T, W>> = entries.into_iter().collect();
        let indices =
            Indices::try_new(entries.iter().map(|pair| pair.weight.clone()))?;
        Ok(Self { entries, indices })
    }

    /// Returns a reference to random indices generator.
    pub fn indices(&self) -> &Indices<W> {
        &self.indices
    }

    /// Returns reference to all entries.
    pub fn entries(&self) -> &[Entry<T, W>] {
        &self.entries
    }
}

impl<'rand, I, T, W> NoiseProcessor<I, &'rand Entry<T, W>>
    for &'rand Entries<T, W>
where
    W: Weight,
    I: NoiseInput,
{
    fn process(&self, input: I, generator: &NoiseGen) -> &'rand Entry<T, W> {
        &self.entries[self.indices.process(input, generator)]
    }
}

impl<'rand, T, W> Distribution<&'rand Entry<T, W>> for &'rand Entries<T, W>
where
    W: Weight + SampleUniform,
{
    fn sample<R>(&self, rng: &mut R) -> &'rand Entry<T, W>
    where
        R: Rng + ?Sized,
    {
        &self.entries[self.indices.sample(rng)]
    }
}

/// Iterator over a absolute value function of weights.
#[derive(Debug)]
pub struct TentWeightFn<I, W>
where
    I: Iterator,
    W: Weight,
{
    iter: I,
    low: W,
    high: W,
    peak: usize,
    size: usize,
    curr: usize,
}

impl<I, W> TentWeightFn<I, W>
where
    I: Iterator,
    W: Weight,
{
    /// Creates an iterator over linear growth and linear decrease of weights,
    /// in a fashion of tent map functions.
    ///
    /// The range of weights is `[low, high)` (high excluded). Peak is the index
    /// of the peak of the weight function
    ///
    /// # Panics
    /// Panics if `low >= high`.
    pub fn new<J>(iter: J, low: W, high: W, peak: usize) -> Self
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

impl<T, W> TentWeightFn<NumRange<T>, W>
where
    T: Ord + NumAssign + Clone + ToPrimitive,
    W: Weight,
{
    /// Creates an iterator over linear growth and linear decrease of weights,
    /// in a fashion of tent map functions. `FromPrimitive::from_usize` is
    /// required in ordeer to use this method.
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
        let high = W::from_usize(half).expect("from_usize is required");
        Self::new(range, W::one(), high + W::one(), half)
    }
}

impl<I, W> Iterator for TentWeightFn<I, W>
where
    W: Weight,
    I: Iterator,
{
    type Item = Entry<I::Item, W>;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.iter.next()?;
        let curr = W::from_usize(self.curr).expect("from_usize required");
        let size = W::from_usize(self.size).expect("from_usize required");
        let range = self.high.clone() - self.low.clone();
        let ratio = Ratio::new(range, size);
        let scale = Ratio::new(curr, W::one());
        let offset = (ratio * scale).to_integer();
        let weight = self.low.clone() + offset;
        self.curr += 1;
        Some(Entry { data, weight })
    }
}

#[cfg(test)]
mod test {
    use super::Indices;

    #[test]
    fn sum_of_weights() {
        let indices = Indices::new([1, 3, 8].iter().cloned());
        assert_eq!(&indices.sums, &[1, 4, 12]);
    }
}
