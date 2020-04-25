/// Random number generation utilites.
pub mod rand;

/// Exports coordinates related items, such as [math::plane::Axis],
/// [math::plane::Point], etc.
pub mod plane;

use num::traits::{cast::ToPrimitive, NumAssign};

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
