use std::ops::{Add, Sub};

use crate::coord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vector<T, const N: usize> {
    pub coords: [T; N],
}

impl<T, const N: usize> Default for Vector<T, { N }>
where
    T: Default,
{
    fn default() -> Self {
        Self { coords: coord::from_fn(|_| T::default()) }
    }
}

impl<T, const N: usize> Add<Self> for Vector<T, { N }>
where
    T: Add,
{
    type Output = Vector<T::Output, { N }>;

    fn add(self, rhs: Self) -> Self::Output {
        Vector {
            coords: coord::zip_with(self.coords, rhs.coords, |a, b| a + b),
        }
    }
}

impl<T, const N: usize> Sub<Self> for Vector<T, { N }>
where
    T: Sub,
{
    type Output = Vector<T::Output, { N }>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector {
            coords: coord::zip_with(self.coords, rhs.coords, |a, b| a - b),
        }
    }
}
