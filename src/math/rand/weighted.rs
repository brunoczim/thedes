use crate::{
    error::Result,
    math::rand::noise::{NoiseGen, NoiseInput, NoiseProcessor},
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
    Ratio<Self>: FromPrimitive,
{
}

impl<W> Weight for W
where
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
        + Clone,
    Ratio<Self>: FromPrimitive,
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
    Ratio<W>: FromPrimitive,
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

#[derive(Debug, Clone)]
pub struct RandomIndices<W> {
    sums: Vec<W>,
}

impl<W> RandomIndices<W>
where
    W: Weight,
    Ratio<W>: FromPrimitive,
{
    /// Builds a new weighted random generator.
    ///
    /// # Panics
    /// Panics if weights overflow.
    pub fn new<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = W>,
    {
        Self::try_new(entries).expect("RandomEntries::new failed")
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

impl<I, W> NoiseProcessor<I, usize> for RandomIndices<W>
where
    W: Weight,
    Ratio<W>: FromPrimitive,
    I: NoiseInput,
{
    fn process(&self, input: I, generator: &NoiseGen) -> usize {
        let noise_raw = generator.gen(input);
        let noise = Ratio::from_f64(noise_raw).expect("from_f64 is required ");
        let scale = self.sums.last().expect("checked on new").clone();
        let scaled = noise * Ratio::new(scale, W::one());
        let search = scaled.to_integer() + W::one();
        self.sums
            .binary_search(&search)
            .unwrap_or_else(|index| index.min(self.sums.len() - 1))
    }
}

impl<W> Distribution<usize> for RandomIndices<W>
where
    W: Weight,
    Ratio<W>: FromPrimitive,
    W: SampleUniform,
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
#[derive(Debug, Clone)]
pub struct RandomEntries<T, W>
where
    Ratio<W>: FromPrimitive,
    W: Weight,
    Ratio<W>: FromPrimitive,
{
    entries: Vec<Entry<T, W>>,
    indices: RandomIndices<W>,
}

impl<T, W> RandomEntries<T, W>
where
    W: Weight,
    Ratio<W>: FromPrimitive,
{
    /// Builds a new weighted random generator.
    ///
    /// # Panics
    /// Panics if weights overflow.
    pub fn new<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = Entry<T, W>>,
    {
        Self::try_new(entries).expect("RandomEntries::new failed")
    }

    /// Builds a new weighted random generator. Returns error if weights
    /// overflow or if sum of weights are zero.
    pub fn try_new<I>(entries: I) -> Result<Self>
    where
        I: IntoIterator<Item = Entry<T, W>>,
    {
        let entries: Vec<Entry<T, W>> = entries.into_iter().collect();
        let indices = RandomIndices::try_new(
            entries.iter().map(|pair| pair.weight.clone()),
        )?;
        Ok(Self { entries, indices })
    }

    /// Returns a reference to random indices generator.
    pub fn indices(&self) -> &RandomIndices<W> {
        &self.indices
    }

    /// Returns reference to all entries.
    pub fn entries(&self) -> &[Entry<T, W>] {
        &self.entries
    }
}

impl<'rand, I, T, W> NoiseProcessor<I, &'rand Entry<T, W>>
    for &'rand RandomEntries<T, W>
where
    W: Weight,
    Ratio<W>: FromPrimitive,
    I: NoiseInput,
{
    fn process(&self, input: I, generator: &NoiseGen) -> &'rand Entry<T, W> {
        &self.entries[self.indices.process(input, generator)]
    }
}

impl<'rand, T, W> Distribution<&'rand Entry<T, W>>
    for &'rand RandomEntries<T, W>
where
    W: Weight,
    Ratio<W>: FromPrimitive,
    W: SampleUniform,
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
    Ratio<W>: FromPrimitive,
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
    Ratio<W>: FromPrimitive,
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
    Ratio<W>: FromPrimitive,
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
    Ratio<W>: FromPrimitive,
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

/// Iterator over any number-typed range.
#[derive(Debug, Clone)]
pub struct NumRange<T>
where
    T: Ord + NumAssign + Clone + ToPrimitive,
{
    /// Inclusive start,
    pub start: T,
    /// Inclusive start,
    pub end: T,
}

impl<T> Iterator for NumRange<T>
where
    T: Ord + NumAssign + Clone + ToPrimitive,
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
    T: Ord + NumAssign + Clone + ToPrimitive
{
}

#[cfg(test)]
mod test {
    use super::RandomIndices;

    #[test]
    fn sum_of_weights() {
        let indices = RandomIndices::new([1, 3, 8].iter().cloned());
        assert_eq!(&indices.sums, &[1, 4, 12]);
    }
}
