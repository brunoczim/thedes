use crate::coord::{Axis, Coord2, Direc, Nat, Rect};
use rand::{distributions::Distribution, Rng};

/// Rectangular houses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RectHouse {
    /// The rectangle occupied by this house.
    pub rect: Rect,
    /// The door coordinates of this house.
    pub door: Coord2<Nat>,
}

/// Uniform distribution of rectangle houses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RectDistribution {
    /// Low (inclusive) coordinate limit.
    pub low_limit: Coord2<Nat>,
    /// High (exclusive) coordinate limit.
    pub high_limit: Coord2<Nat>,
    /// Minimum rectangle size.
    pub min_size: Coord2<Nat>,
    /// Maximum rectangle size.
    pub max_size: Coord2<Nat>,
}

impl Distribution<RectHouse> for RectDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RectHouse {
        tracing::debug!(?self);
        let start = self.low_limit.zip(self.high_limit).zip_with(
            self.min_size,
            |(low, high), min_size| {
                rng.gen_range(low, high.min(high - min_size))
            },
        );

        let size = self.max_size.zip(self.min_size).zip_with(
            start.zip(self.high_limit),
            |(max_size, min_size), (start, limit)| {
                rng.gen_range(min_size, max_size.min(limit - start))
            },
        );

        let rect = Rect { start, size };

        let (fixed_axis, limit) = match rng.gen() {
            Direc::Up => (Axis::Y, rect.start),
            Direc::Left => (Axis::X, rect.start),
            Direc::Down => (Axis::Y, rect.end().map(|val| val - 1)),
            Direc::Right => (Axis::X, rect.end().map(|val| val - 1)),
        };

        let mut door = Coord2 { x: 0, y: 0 };
        door[fixed_axis] = limit[fixed_axis];
        door[!fixed_axis] = rng.gen_range(
            rect.start[!fixed_axis] + 1,
            rect.end()[!fixed_axis] - 1,
        );

        RectHouse { rect, door }
    }
}
