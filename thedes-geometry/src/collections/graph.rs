use std::{
    borrow::Borrow,
    fmt,
    mem,
    ops::{Add, Bound, Sub},
};

use crate::{
    coords::CoordRange,
    orientation::{Axis, Direction, DirectionFlags},
    CoordPair,
};

use super::{map, CoordMap};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum ConnectError<C> {
    UnknownNode(CoordPair<C>),
    NoStraightDirection(CoordPair<C>, CoordPair<C>),
}

impl<C> fmt::Display for ConnectError<C>
where
    C: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode(location) => {
                write!(f, "node location {location} is unknown")
            },
            Self::NoStraightDirection(a, b) => write!(
                f,
                "nodes with locations {a} and {b} do not have a straight \
                 direction"
            ),
        }
    }
}

impl<C> std::error::Error for ConnectError<C> where C: fmt::Display + fmt::Debug {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Node {
    edges: DirectionFlags,
}

impl Node {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connected(self, direction: Direction) -> bool {
        self.edges.map()[direction]
    }

    pub fn connect(&mut self, direction: Direction) -> bool {
        !self.edges.modify(|map| mem::replace(&mut map[direction], true))
    }

    pub fn with_connection(mut self, direction: Direction) -> Self {
        self.connect(direction);
        self
    }

    pub fn disconnect(&mut self, direction: Direction) -> bool {
        self.edges.modify(|map| mem::replace(&mut map[direction], false))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoordDiGraph<C> {
    nodes: CoordMap<C, Node>,
}

impl<C> CoordDiGraph<C> {
    pub fn new() -> Self {
        Self { nodes: CoordMap::default() }
    }
}

impl<C> CoordDiGraph<C>
where
    C: Ord,
{
    pub fn contains_node<C0>(&self, location: CoordPair<&C0>) -> bool
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        self.node(location).is_some()
    }

    pub fn node<C0>(&self, location: CoordPair<&C0>) -> Option<Node>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        let (_, node) = self.node_with_location(location)?;
        Some(node)
    }

    pub fn node_with_location<C0>(
        &self,
        location: CoordPair<&C0>,
    ) -> Option<(CoordPair<&C>, Node)>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        let (found_location, node) = self.nodes.get_key_value(location)?;
        Some((found_location, *node))
    }

    pub fn neighbors<'q, 'a, C0>(
        &'a self,
        location: CoordPair<&'q C0>,
        direction: Direction,
    ) -> Neighbors<'q, 'a, C0, C>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        match direction {
            Direction::Up => Neighbors {
                rev: true,
                inner: self.nodes.range(
                    Axis::Y,
                    CoordRange {
                        y: .. location.y,
                        x: location.x ..= location.x,
                    }
                    .to_bounds(),
                ),
            },
            Direction::Left => Neighbors {
                rev: true,
                inner: self.nodes.range(
                    Axis::X,
                    CoordRange {
                        y: location.y ..= location.y,
                        x: .. location.x,
                    }
                    .to_bounds(),
                ),
            },
            Direction::Down => Neighbors {
                rev: false,
                inner: self.nodes.range(
                    Axis::Y,
                    CoordRange {
                        y: (Bound::Excluded(location.y), Bound::Unbounded),
                        x: location.x ..= location.x,
                    }
                    .to_bounds(),
                ),
            },
            Direction::Right => Neighbors {
                rev: false,
                inner: self.nodes.range(
                    Axis::X,
                    CoordRange {
                        y: location.y ..= location.y,
                        x: (Bound::Excluded(location.x), Bound::Unbounded),
                    }
                    .to_bounds(),
                ),
            },
        }
    }

    pub fn neighbors_inclusive<'q, 'a, C0>(
        &'a self,
        location: CoordPair<&'q C0>,
        direction: Direction,
    ) -> Neighbors<'q, 'a, C0, C>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        match direction {
            Direction::Up => Neighbors {
                rev: true,
                inner: self.nodes.range(
                    Axis::Y,
                    CoordRange {
                        y: ..= location.y,
                        x: location.x ..= location.x,
                    }
                    .to_bounds(),
                ),
            },
            Direction::Left => Neighbors {
                rev: true,
                inner: self.nodes.range(
                    Axis::X,
                    CoordRange {
                        y: location.y ..= location.y,
                        x: ..= location.x,
                    }
                    .to_bounds(),
                ),
            },
            Direction::Down => Neighbors {
                rev: false,
                inner: self.nodes.range(
                    Axis::Y,
                    CoordRange {
                        y: location.y ..,
                        x: location.x ..= location.x,
                    }
                    .to_bounds(),
                ),
            },
            Direction::Right => Neighbors {
                rev: false,
                inner: self.nodes.range(
                    Axis::X,
                    CoordRange {
                        y: location.y ..= location.y,
                        x: location.x ..,
                    }
                    .to_bounds(),
                ),
            },
        }
    }
}

impl<C> CoordDiGraph<C>
where
    C: Ord + Clone,
{
    pub fn insert_node(&mut self, node: CoordPair<C>) -> bool {
        match self.nodes.entry(node) {
            map::Entry::Occupied(_) => false,
            map::Entry::Vacant(entry) => {
                entry.insert(Node::default());
                true
            },
        }
    }
}

impl<C> CoordDiGraph<C>
where
    C: Ord + Clone + Add<Output = C> + Sub<Output = C>,
{
    pub fn remove_node(
        &mut self,
        node: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        for direction in Direction::ALL {
            let Some(_) = self
                .nodes
                .with_mut(node.as_ref(), |node| node.disconnect(direction))
            else {
                return Ok(false);
            };
            if self.neighbors(node.as_ref(), direction).next().is_none() {
                if let Some(neighbor) = self
                    .neighbors(node.as_ref(), -direction)
                    .next()
                    .map(|(coords, _)| coords.cloned())
                {
                    self.nodes.with_mut(neighbor.as_ref(), |node| {
                        node.disconnect(direction)
                    });
                }
            }
        }
        self.nodes.remove(node.as_ref());
        Ok(true)
    }

    pub fn connected(
        &self,
        origin: CoordPair<C>,
        destiny: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        if !self.contains_node(origin.as_ref()) {
            return Err(ConnectError::UnknownNode(origin));
        }
        if !self.contains_node(destiny.as_ref()) {
            return Err(ConnectError::UnknownNode(destiny));
        }
        let Some(vector) = origin.clone().direction_to(destiny.clone()) else {
            return Err(ConnectError::NoStraightDirection(origin, destiny));
        };
        let mut neighbors =
            self.neighbors_inclusive(origin.as_ref(), vector.direction);
        loop {
            let Some((neighbor, node)) = neighbors.next() else {
                break Ok(true);
            };
            if neighbor == destiny.as_ref() {
                break Ok(true);
            }
            if !node.connected(vector.direction) {
                break Ok(false);
            }
        }
    }

    pub fn undirected_connected(
        &self,
        origin: CoordPair<C>,
        destiny: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        if !self.contains_node(origin.as_ref()) {
            return Err(ConnectError::UnknownNode(origin));
        }
        if !self.contains_node(destiny.as_ref()) {
            return Err(ConnectError::UnknownNode(destiny));
        }
        let Some(vector) = origin.clone().direction_to(destiny.clone()) else {
            return Err(ConnectError::NoStraightDirection(origin, destiny));
        };
        let mut neighbors =
            self.neighbors_inclusive(origin.as_ref(), vector.direction);
        loop {
            let Some((neighbor, node)) = neighbors.next() else {
                break Ok(true);
            };
            if neighbor != origin.as_ref() && !node.connected(-vector.direction)
            {
                break Ok(false);
            }
            if neighbor == destiny.as_ref() {
                break Ok(true);
            }
            if !node.connected(vector.direction) {
                break Ok(false);
            }
        }
    }

    pub fn connect(
        &mut self,
        origin: CoordPair<C>,
        destiny: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        if !self.contains_node(origin.as_ref()) {
            return Err(ConnectError::UnknownNode(origin));
        }
        if !self.contains_node(destiny.as_ref()) {
            return Err(ConnectError::UnknownNode(destiny));
        }
        let Some(vector) = origin.clone().direction_to(destiny.clone()) else {
            return Err(ConnectError::NoStraightDirection(origin, destiny));
        };
        let mut current = origin;
        let mut new = false;
        while let Some((neighbor, _)) = self
            .neighbors(current.as_ref(), vector.direction)
            .next()
            .filter(|_| current != destiny)
        {
            let next = neighbor.cloned();
            self.nodes.with_mut(current.as_ref(), |node| {
                new |= node.connect(vector.direction);
            });
            current = next;
        }
        Ok(new)
    }

    pub fn connect_undirected(
        &mut self,
        origin: CoordPair<C>,
        destiny: CoordPair<C>,
    ) -> Result<(bool, bool), ConnectError<C>> {
        if !self.contains_node(origin.as_ref()) {
            return Err(ConnectError::UnknownNode(origin));
        }
        if !self.contains_node(destiny.as_ref()) {
            return Err(ConnectError::UnknownNode(destiny));
        }
        let Some(vector) = origin.clone().direction_to(destiny.clone()) else {
            return Err(ConnectError::NoStraightDirection(origin, destiny));
        };
        let mut current = origin;
        let mut new = false;
        let mut new_rev = false;
        while let Some((neighbor, _)) = self
            .neighbors(current.as_ref(), vector.direction)
            .next()
            .filter(|_| current != destiny)
        {
            let next = neighbor.cloned();
            self.nodes.with_mut(current.as_ref(), |node| {
                new |= node.connect(vector.direction);
            });
            self.nodes.with_mut(next.as_ref(), |node| {
                new_rev |= node.connect(-vector.direction);
            });
            current = next;
        }
        Ok((new, new_rev))
    }

    pub fn disconnect(
        &mut self,
        origin: CoordPair<C>,
        destiny: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        if !self.connected(origin.clone(), destiny.clone())? {
            return Ok(false);
        }
        let Some(vector) = origin.clone().direction_to(destiny.clone()) else {
            return Err(ConnectError::NoStraightDirection(origin, destiny));
        };
        let mut current = origin;
        while let Some((neighbor, _)) = self
            .neighbors(current.as_ref(), vector.direction)
            .next()
            .filter(|_| current != destiny)
        {
            let next = neighbor.cloned();
            self.nodes.with_mut(current.as_ref(), |node| {
                node.disconnect(vector.direction);
            });
            current = next;
        }
        Ok(true)
    }

    pub fn disconnect_undirected(
        &mut self,
        origin: CoordPair<C>,
        destiny: CoordPair<C>,
    ) -> Result<(bool, bool), ConnectError<C>> {
        if !self.connected(origin.clone(), destiny.clone())?
            && !self.connected(destiny.clone(), origin.clone())?
        {
            return Ok((false, false));
        }
        let Some(vector) = origin.clone().direction_to(destiny.clone()) else {
            return Err(ConnectError::NoStraightDirection(origin, destiny));
        };
        let mut current = origin;
        let mut new = false;
        let mut new_rev = false;
        while let Some((neighbor, _)) = self
            .neighbors(current.as_ref(), vector.direction)
            .next()
            .filter(|_| current != destiny)
        {
            let next = neighbor.cloned();
            self.nodes.with_mut(current.as_ref(), |node| {
                new |= node.disconnect(vector.direction);
            });
            self.nodes.with_mut(next.as_ref(), |node| {
                new_rev |= node.disconnect(-vector.direction);
            });
            current = next;
        }
        Ok((new, new_rev))
    }
}

#[derive(Debug, Clone)]
pub struct Neighbors<'q, 'a, C0, C> {
    inner: map::Range<'q, 'a, C0, C, Node>,
    rev: bool,
}

impl<'q, 'a, C0, C> Iterator for Neighbors<'q, 'a, C0, C>
where
    C: Ord + Borrow<C0>,
    C0: Ord,
{
    type Item = (CoordPair<&'a C>, Node);

    fn next(&mut self) -> Option<Self::Item> {
        let (location, node) = if self.rev {
            self.inner.next_back()?
        } else {
            self.inner.next()?
        };
        Some((location, *node))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'q, 'a, C0, C> DoubleEndedIterator for Neighbors<'q, 'a, C0, C>
where
    C: Ord + Borrow<C0>,
    C0: Ord,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let (location, node) = if self.rev {
            self.inner.next()?
        } else {
            self.inner.next_back()?
        };
        Some((location, *node))
    }
}
