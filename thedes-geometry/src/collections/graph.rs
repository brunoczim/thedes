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
pub struct CoordGraph<C> {
    inner: CoordDiGraph<C>,
}

impl<C> Default for CoordGraph<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> CoordGraph<C> {
    pub fn new() -> Self {
        Self { inner: CoordDiGraph::new() }
    }
}

impl<C> CoordGraph<C>
where
    C: Ord,
{
    pub fn contains_node<C0>(&self, location: CoordPair<&C0>) -> bool
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        self.inner.contains_node(location)
    }

    pub fn node<C0>(&self, location: CoordPair<&C0>) -> Option<Node>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        self.inner.node(location)
    }

    pub fn node_with_location<C0>(
        &self,
        location: CoordPair<&C0>,
    ) -> Option<(CoordPair<&C>, Node)>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        self.inner.node_with_location(location)
    }
}

impl<C> CoordGraph<C>
where
    C: Ord + Clone,
{
    pub fn insert_node(&mut self, node: CoordPair<C>) -> bool {
        self.inner.insert_node(node)
    }
}

impl<C> CoordGraph<C>
where
    C: Ord + Clone + Add<Output = C> + Sub<Output = C>,
{
    pub fn remove_node(
        &mut self,
        location: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        self.inner.remove_node_undirected(location)
    }

    pub fn connected(
        &self,
        origin: CoordPair<C>,
        destiny: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        self.inner.connected(origin, destiny)
    }

    pub fn connect(
        &mut self,
        origin: CoordPair<C>,
        destiny: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        self.inner.connect_undirected(origin, destiny).map(|(a, b)| a || b)
    }

    pub fn disconnect(
        &mut self,
        origin: CoordPair<C>,
        destiny: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        self.inner.disconnect_undirected(origin, destiny).map(|(a, b)| a || b)
    }
}

impl<C> CoordGraph<C> {
    pub fn iter(&self, higher_axis: Axis) -> Iter<C> {
        self.inner.iter(higher_axis)
    }

    pub fn rows(&self) -> Iter<C> {
        self.iter(Axis::Y)
    }

    pub fn columns(&self) -> Iter<C> {
        self.iter(Axis::X)
    }

    pub fn locations(&self, higher_axis: Axis) -> Locations<C> {
        self.inner.locations(higher_axis)
    }

    pub fn location_rows(&self) -> Locations<C> {
        self.locations(Axis::Y)
    }

    pub fn location_columns(&self) -> Locations<C> {
        self.locations(Axis::X)
    }

    pub fn nodes(&self, higher_axis: Axis) -> Nodes<C> {
        self.inner.nodes(higher_axis)
    }

    pub fn node_rows(&self) -> Nodes<C> {
        self.nodes(Axis::Y)
    }

    pub fn node_columns(&self) -> Nodes<C> {
        self.nodes(Axis::X)
    }
}

impl<C> CoordGraph<C>
where
    C: Clone,
{
    pub fn into_iter_with(self, higher_axis: Axis) -> IntoIter<C> {
        self.inner.into_iter_with(higher_axis)
    }

    pub fn into_rows(self) -> IntoIter<C> {
        self.into_iter_with(Axis::Y)
    }

    pub fn into_columns(self) -> IntoIter<C> {
        self.into_iter_with(Axis::X)
    }

    pub fn into_locations(self, higher_axis: Axis) -> IntoLocations<C> {
        self.inner.into_locations(higher_axis)
    }

    pub fn into_location_rows(self) -> IntoLocations<C> {
        self.into_locations(Axis::Y)
    }

    pub fn into_location_columns(self) -> IntoLocations<C> {
        self.into_locations(Axis::X)
    }
}

impl<C> CoordGraph<C> {
    pub fn into_nodes(self, higher_axis: Axis) -> IntoNodes<C> {
        self.inner.into_nodes(higher_axis)
    }

    pub fn into_node_rows(self) -> IntoNodes<C> {
        self.into_nodes(Axis::Y)
    }

    pub fn into_node_columns(self) -> IntoNodes<C> {
        self.into_nodes(Axis::X)
    }
}

impl<C> CoordGraph<C>
where
    C: Ord,
{
    pub fn neighbors<'q, 'a, C0>(
        &'a self,
        location: CoordPair<&'q C0>,
        direction: Direction,
    ) -> Neighbors<'q, 'a, C0, C>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        self.inner.neighbors(location, direction)
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
        self.inner.neighbors_inclusive(location, direction)
    }
}

impl<C> IntoIterator for CoordGraph<C>
where
    C: Clone,
{
    type IntoIter = IntoIter<C>;
    type Item = (CoordPair<C>, Node);

    fn into_iter(self) -> Self::IntoIter {
        self.into_rows()
    }
}

impl<'a, C> IntoIterator for &'a CoordGraph<C>
where
    C: Clone,
{
    type IntoIter = Iter<'a, C>;
    type Item = (CoordPair<&'a C>, Node);

    fn into_iter(self) -> Self::IntoIter {
        self.rows()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoordDiGraph<C> {
    nodes: CoordMap<C, Node>,
}

impl<C> Default for CoordDiGraph<C> {
    fn default() -> Self {
        Self::new()
    }
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
        location: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        for direction in Direction::ALL {
            let Some(_) = self
                .nodes
                .with_mut(location.as_ref(), |node| node.disconnect(direction))
            else {
                return Ok(false);
            };
            if self.neighbors(location.as_ref(), direction).next().is_none() {
                if let Some(neighbor) = self
                    .neighbors(location.as_ref(), -direction)
                    .next()
                    .map(|(coords, _)| coords.cloned())
                {
                    self.nodes.with_mut(neighbor.as_ref(), |node| {
                        node.disconnect(direction)
                    });
                }
            }
        }
        self.nodes.remove(location.as_ref());
        Ok(true)
    }

    pub fn remove_node_undirected(
        &mut self,
        location: CoordPair<C>,
    ) -> Result<bool, ConnectError<C>> {
        // use case:
        //
        // A <-> B . C
        //
        // remove(B)
        //
        // expected:
        //
        // A . C
        //
        // unexpected:
        //
        // A -> C
        for direction in Direction::ALL {
            let Some(disconnected) = self
                .nodes
                .with_mut(location.as_ref(), |node| node.disconnect(direction))
            else {
                return Ok(false);
            };
            if self.neighbors(location.as_ref(), direction).next().is_none()
                || !disconnected
            {
                if let Some(neighbor) = self
                    .neighbors(location.as_ref(), -direction)
                    .next()
                    .map(|(coords, _)| coords.cloned())
                {
                    self.nodes.with_mut(neighbor.as_ref(), |node| {
                        node.disconnect(direction)
                    });
                }
            }
        }
        self.nodes.remove(location.as_ref());
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

impl<C> CoordDiGraph<C> {
    pub fn iter(&self, higher_axis: Axis) -> Iter<C> {
        Iter { inner: self.nodes.iter(higher_axis) }
    }

    pub fn rows(&self) -> Iter<C> {
        self.iter(Axis::Y)
    }

    pub fn columns(&self) -> Iter<C> {
        self.iter(Axis::X)
    }

    pub fn locations(&self, higher_axis: Axis) -> Locations<C> {
        Locations { inner: self.nodes.keys(higher_axis) }
    }

    pub fn location_rows(&self) -> Locations<C> {
        self.locations(Axis::Y)
    }

    pub fn location_columns(&self) -> Locations<C> {
        self.locations(Axis::X)
    }

    pub fn nodes(&self, higher_axis: Axis) -> Nodes<C> {
        Nodes { inner: self.nodes.values(higher_axis) }
    }

    pub fn node_rows(&self) -> Nodes<C> {
        self.nodes(Axis::Y)
    }

    pub fn node_columns(&self) -> Nodes<C> {
        self.nodes(Axis::X)
    }
}

impl<C> CoordDiGraph<C>
where
    C: Clone,
{
    pub fn into_iter_with(self, higher_axis: Axis) -> IntoIter<C> {
        IntoIter { inner: self.nodes.into_iter_with(higher_axis) }
    }

    pub fn into_rows(self) -> IntoIter<C> {
        self.into_iter_with(Axis::Y)
    }

    pub fn into_columns(self) -> IntoIter<C> {
        self.into_iter_with(Axis::X)
    }

    pub fn into_locations(self, higher_axis: Axis) -> IntoLocations<C> {
        IntoLocations { inner: self.nodes.into_keys(higher_axis) }
    }

    pub fn into_location_rows(self) -> IntoLocations<C> {
        self.into_locations(Axis::Y)
    }

    pub fn into_location_columns(self) -> IntoLocations<C> {
        self.into_locations(Axis::X)
    }
}

impl<C> CoordDiGraph<C> {
    pub fn into_nodes(self, higher_axis: Axis) -> IntoNodes<C> {
        IntoNodes { inner: self.nodes.into_values(higher_axis) }
    }

    pub fn into_node_rows(self) -> IntoNodes<C> {
        self.into_nodes(Axis::Y)
    }

    pub fn into_node_columns(self) -> IntoNodes<C> {
        self.into_nodes(Axis::X)
    }
}

impl<C> CoordDiGraph<C>
where
    C: Ord,
{
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

impl<C> IntoIterator for CoordDiGraph<C>
where
    C: Clone,
{
    type IntoIter = IntoIter<C>;
    type Item = (CoordPair<C>, Node);

    fn into_iter(self) -> Self::IntoIter {
        self.into_rows()
    }
}

impl<'a, C> IntoIterator for &'a CoordDiGraph<C>
where
    C: Clone,
{
    type IntoIter = Iter<'a, C>;
    type Item = (CoordPair<&'a C>, Node);

    fn into_iter(self) -> Self::IntoIter {
        self.rows()
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

#[derive(Debug, Clone)]
pub struct Iter<'a, C> {
    inner: map::Iter<'a, C, Node>,
}

impl<'a, C> Iterator for Iter<'a, C> {
    type Item = (CoordPair<&'a C>, Node);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(location, node)| (location, *node))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, C> DoubleEndedIterator for Iter<'a, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(location, node)| (location, *node))
    }
}

#[derive(Debug, Clone)]
pub struct Locations<'a, C> {
    inner: map::Keys<'a, C, Node>,
}

impl<'a, C> Iterator for Locations<'a, C> {
    type Item = CoordPair<&'a C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, C> DoubleEndedIterator for Locations<'a, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

#[derive(Debug, Clone)]
pub struct Nodes<'a, C> {
    inner: map::Values<'a, C, Node>,
}

impl<'a, C> Iterator for Nodes<'a, C> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, C> DoubleEndedIterator for Nodes<'a, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().copied()
    }
}

#[derive(Debug)]
pub struct IntoIter<C> {
    inner: map::IntoIter<C, Node>,
}

impl<C> Iterator for IntoIter<C>
where
    C: Clone,
{
    type Item = (CoordPair<C>, Node);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<C> DoubleEndedIterator for IntoIter<C>
where
    C: Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

#[derive(Debug)]
pub struct IntoLocations<C> {
    inner: map::IntoKeys<C, Node>,
}

impl<C> Iterator for IntoLocations<C>
where
    C: Clone,
{
    type Item = CoordPair<C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<C> DoubleEndedIterator for IntoLocations<C>
where
    C: Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

#[derive(Debug)]
pub struct IntoNodes<C> {
    inner: map::IntoValues<C, Node>,
}

impl<C> Iterator for IntoNodes<C> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<C> DoubleEndedIterator for IntoNodes<C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}
