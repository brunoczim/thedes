use std::{
    iter::{Skip, Take},
    ops::Range,
};

/// Extension to std's iterator trait.
pub trait IterExt: Iterator {
    /// Gets a slice of iterated items.
    fn slice(self, bounds: Range<usize>) -> Take<Skip<Self>>
    where
        Self: Sized,
    {
        self.skip(bounds.start).take(bounds.end - bounds.start)
    }
}

impl<I> IterExt for I where I: Iterator {}
