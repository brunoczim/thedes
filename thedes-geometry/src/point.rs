use crate::{coord, vector::Vector};
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point<T, const N: usize> {
    pub coords: [T; N],
}

impl<T, const N: usize> Default for Point<T, { N }>
where
    T: Default,
{
    fn default() -> Self {
        Self { coords: coord::from_fn(|_| T::default()) }
    }
}

impl<T, const N: usize> Add<Vector<T, { N }>> for Point<T, { N }>
where
    T: Add,
{
    type Output = Point<T::Output, { N }>;

    fn add(self, rhs: Vector<T, { N }>) -> Self::Output {
        Point { coords: coord::zip_with(self.coords, rhs.coords, |a, b| a + b) }
    }
}

impl<T, const N: usize> Sub<Self> for Point<T, { N }>
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

impl<T, const N: usize> Add<T> for Point<T, { N }>
where
    T: Add + Clone,
{
    type Output = Point<T::Output, { N }>;

    fn add(self, rhs: T) -> Self::Output {
        Point { coords: self.coords.map(|a| a + rhs.clone()) }
    }
}
