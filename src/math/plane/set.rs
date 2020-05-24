use crate::math::plane::{Axis, Coord2, Direc, Nat};
use std::{
    collections::{btree_set, BTreeSet},
    ops::Bound,
};

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
    pub fn new() -> Self {
        Set { neighbours: Coord2::from_axes(|_| BTreeSet::new()) }
    }

    pub fn contains(&self, point: Coord2<Nat>) -> bool {
        self.neighbours.x.contains(&point)
    }

    pub fn neighbour(
        &self,
        point: Coord2<Nat>,
        direc: Direc,
    ) -> Option<Coord2<Nat>> {
        let (axis, start, end) = match direc {
            Direc::Up => (
                Axis::Y,
                Bound::Included(!Coord2 { y: 0, ..point }),
                Bound::Excluded(!point),
            ),
            Direc::Left => (
                Axis::X,
                Bound::Included(Coord2 { x: 0, ..point }),
                Bound::Excluded(point),
            ),
            Direc::Down => (
                Axis::Y,
                Bound::Excluded(!point),
                Bound::Included(!Coord2 { y: Nat::max_value(), ..point }),
            ),
            Direc::Right => (
                Axis::X,
                Bound::Excluded(point),
                Bound::Included(Coord2 { x: Nat::max_value(), ..point }),
            ),
        };

        self.neighbours[axis].range((start, end)).next().map(Clone::clone)
    }

    pub fn insert(&mut self, point: Coord2<Nat>) {
        self.neighbours.x.insert(point);
        self.neighbours.y.insert(!point);
    }

    pub fn remove(&mut self, point: Coord2<Nat>) -> bool {
        self.neighbours.x.remove(&point) && self.neighbours.y.remove(&!point)
    }

    pub fn rows(&self) -> Rows {
        Rows { inner: self.neighbours.x.iter() }
    }

    pub fn columns(&self) -> Columns {
        Columns { inner: self.neighbours.y.iter() }
    }
}

#[derive(Debug, Clone)]
pub struct Rows<'set> {
    inner: btree_set::Iter<'set, Coord2<Nat>>,
}

impl<'set> Iterator for Rows<'set> {
    type Item = Coord2<Nat>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Clone::clone)
    }
}

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
