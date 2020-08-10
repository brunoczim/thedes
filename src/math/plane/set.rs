use crate::math::plane::{Coord2, Direc, Nat};
use std::{
    collections::{btree_set, BTreeSet},
    ops::Bound,
};

/// A set of coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Set {
    neighbours: Coord2<BTreeSet<Coord2<Nat>>>,
}

impl Default for Set {
    fn default() -> Self {
        Self::new()
    }
}

impl Set {
    /// Creates an empty set of coordinates;
    pub fn new() -> Self {
        Set { neighbours: Coord2::from_axes(|_| BTreeSet::new()) }
    }

    /// Count of coordinates in the set.
    pub fn len(&self) -> usize {
        self.neighbours.x.len()
    }

    /// Tests whether a given point is in the set.
    pub fn contains(&self, point: Coord2<Nat>) -> bool {
        self.neighbours.x.contains(&point)
    }

    /// Searches for the neighbour of the given point in the given
    /// direction.
    ///
    /// A neighbour is the closest neighbour in that direction, with the same X
    /// or Y (which depends on the direction).
    pub fn neighbour(
        &self,
        point: Coord2<Nat>,
        direc: Direc,
    ) -> Option<Coord2<Nat>> {
        match direc {
            Direc::Up => self
                .neighbours
                .y
                .range(!Coord2 { y: 0, ..point } .. !point)
                .map(|&point| !point)
                .next_back(),
            Direc::Left => self
                .neighbours
                .x
                .range(Coord2 { x: 0, ..point } .. point)
                .map(|&point| point)
                .next_back(),
            Direc::Down => self
                .neighbours
                .y
                .range((
                    Bound::Excluded(!point),
                    Bound::Included(!Coord2 { y: Nat::max_value(), ..point }),
                ))
                .map(|&point| !point)
                .next(),
            Direc::Right => self
                .neighbours
                .x
                .range((
                    Bound::Excluded(point),
                    Bound::Included(Coord2 { x: Nat::max_value(), ..point }),
                ))
                .map(|&point| point)
                .next(),
        }
    }

    /// Searches for the last neighbour of the given point in the given
    /// direction.
    ///
    /// A neighbour is the closest neighbour in that direction, with the same X
    /// or Y (which depends on the direction).
    pub fn last_neighbour(
        &self,
        point: Coord2<Nat>,
        direc: Direc,
    ) -> Option<Coord2<Nat>> {
        match direc {
            Direc::Up => self
                .neighbours
                .y
                .range(!Coord2 { y: 0, ..point } ..= !point)
                .map(|&point| !point)
                .next(),
            Direc::Left => self
                .neighbours
                .x
                .range(Coord2 { x: 0, ..point } ..= point)
                .map(|&point| point)
                .next(),
            Direc::Down => self
                .neighbours
                .y
                .range(!point ..= !Coord2 { y: Nat::max_value(), ..point })
                .map(|&point| !point)
                .next_back(),
            Direc::Right => self
                .neighbours
                .x
                .range(point ..= Coord2 { x: Nat::max_value(), ..point })
                .map(|&point| point)
                .next_back(),
        }
    }

    /// Inserts a point in the set.
    pub fn insert(&mut self, point: Coord2<Nat>) {
        self.neighbours.x.insert(point);
        self.neighbours.y.insert(!point);
    }

    /// Removes a point in the set.
    pub fn remove(&mut self, point: Coord2<Nat>) -> bool {
        self.neighbours.x.remove(&point) && self.neighbours.y.remove(&!point)
    }

    /// Iterates through the rows (in terms of plane) of points in the set.
    pub fn rows(&self) -> Rows {
        Rows { inner: self.neighbours.x.iter() }
    }

    /// Iterates through the columns (in terms of plane) of points in the set.
    pub fn columns(&self) -> Columns {
        Columns { inner: self.neighbours.y.iter() }
    }
}

/// Iterator over the rows of a set.
#[derive(Debug, Clone)]
pub struct Rows<'set> {
    inner: btree_set::Iter<'set, Coord2<Nat>>,
}

impl<'set> Iterator for Rows<'set> {
    type Item = Coord2<Nat>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|&point| point)
    }
}

/// Iterator over the columns of a set.
#[derive(Debug, Clone)]
pub struct Columns<'set> {
    inner: btree_set::Iter<'set, Coord2<Nat>>,
}

impl<'set> Iterator for Columns<'set> {
    type Item = Coord2<Nat>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|&point| !point)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn neighbour() {
        let mut set = Set::new();
        for x in 5 .. 15 {
            for y in 10 .. 20 {
                let point = Coord2 { x, y };
                if point != (Coord2 { x: 10, y: 14 }) {
                    set.insert(point);
                }
            }
        }

        assert_eq!(
            set.neighbour(Coord2 { x: 10, y: 15 }, Direc::Up).unwrap(),
            Coord2 { x: 10, y: 13 }
        );
        assert_eq!(
            set.neighbour(Coord2 { x: 10, y: 15 }, Direc::Down).unwrap(),
            Coord2 { x: 10, y: 16 }
        );
        assert_eq!(
            set.neighbour(Coord2 { x: 10, y: 15 }, Direc::Left).unwrap(),
            Coord2 { x: 9, y: 15 }
        );
        assert_eq!(
            set.neighbour(Coord2 { x: 10, y: 15 }, Direc::Right).unwrap(),
            Coord2 { x: 11, y: 15 }
        );

        assert_eq!(
            set.last_neighbour(Coord2 { x: 10, y: 15 }, Direc::Up).unwrap(),
            Coord2 { x: 10, y: 10 }
        );
        assert_eq!(
            set.last_neighbour(Coord2 { x: 10, y: 15 }, Direc::Down).unwrap(),
            Coord2 { x: 10, y: 19 }
        );
        assert_eq!(
            set.last_neighbour(Coord2 { x: 10, y: 15 }, Direc::Left).unwrap(),
            Coord2 { x: 5, y: 15 }
        );
        assert_eq!(
            set.last_neighbour(Coord2 { x: 10, y: 15 }, Direc::Right).unwrap(),
            Coord2 { x: 14, y: 15 }
        );
    }
}
