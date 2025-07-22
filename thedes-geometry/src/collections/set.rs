use std::{borrow::Borrow, fmt, marker::PhantomData};

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
};

use crate::{CoordPair, coords::CoordPairBounds, orientation::Axis};

use super::map::{self, CoordMap};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoordSet<C> {
    inner: CoordMap<C, ()>,
}

impl<C> Default for CoordSet<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> CoordSet<C> {
    pub fn new() -> Self {
        Self { inner: CoordMap::new() }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<C> CoordSet<C>
where
    C: Ord,
{
    pub fn contains<C0>(&self, elem: CoordPair<&C0>) -> bool
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        self.get(elem).is_some()
    }

    pub fn get<C0>(&self, elem: CoordPair<&C0>) -> Option<CoordPair<&C>>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        let (stored, _) = self.inner.get_key_value(elem)?;
        Some(stored)
    }
}

impl<C> CoordSet<C>
where
    C: Ord + Clone,
{
    pub fn insert(&mut self, elem: CoordPair<C>) -> bool {
        self.inner.insert(elem, ()).is_none()
    }
}

impl<C> CoordSet<C>
where
    C: Ord,
{
    pub fn remove<C0>(&mut self, elem: CoordPair<&C0>) -> bool
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        self.inner.remove(elem).is_some()
    }

    pub fn range<'q, 'a, R, C0>(
        &'a self,
        higher_axis: Axis,
        ranges: R,
    ) -> Range<'q, 'a, C0, C>
    where
        R: Into<CoordPairBounds<&'q C0>>,
        C: Borrow<C0>,
        C0: Ord + 'q,
    {
        Range { inner: self.inner.range(higher_axis, ranges) }
    }

    pub fn next_neighbor<C0>(
        &self,
        axis: Axis,
        key: CoordPair<&C0>,
    ) -> Option<CoordPair<&C>>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        let (elem, _) = self.inner.next_neighbor(axis, key)?;
        Some(elem)
    }

    pub fn prev_neighbor<C0>(
        &self,
        axis: Axis,
        key: CoordPair<&C0>,
    ) -> Option<CoordPair<&C>>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        let (elem, _) = self.inner.prev_neighbor(axis, key)?;
        Some(elem)
    }

    pub fn last_neighbor<C0>(
        &self,
        axis: Axis,
        key: CoordPair<&C0>,
    ) -> Option<CoordPair<&C>>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        let (elem, _) = self.inner.last_neighbor(axis, key)?;
        Some(elem)
    }

    pub fn first_neighbor<C0>(
        &self,
        axis: Axis,
        key: CoordPair<&C0>,
    ) -> Option<CoordPair<&C>>
    where
        C: Borrow<C0>,
        C0: Ord,
    {
        let (elem, _) = self.inner.first_neighbor(axis, key)?;
        Some(elem)
    }
}

impl<C> CoordSet<C> {
    pub fn iter<'a>(&'a self, higher_axis: Axis) -> Iter<'a, C> {
        Iter { inner: self.inner.keys(higher_axis) }
    }

    pub fn rows<'a>(&'a self) -> Iter<'a, C> {
        self.iter(Axis::Y)
    }

    pub fn columns<'a>(&'a self) -> Iter<'a, C> {
        self.iter(Axis::X)
    }
}

impl<C> CoordSet<C>
where
    C: Clone,
{
    pub fn into_iter_with(self, higher_axis: Axis) -> IntoIter<C> {
        IntoIter { inner: self.inner.into_keys(higher_axis) }
    }

    pub fn into_rows(self) -> IntoIter<C> {
        self.into_iter_with(Axis::Y)
    }

    pub fn into_columns(self) -> IntoIter<C> {
        self.into_iter_with(Axis::X)
    }
}

impl<C> Serialize for CoordSet<C>
where
    C: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut set_serializer = serializer.serialize_seq(Some(self.len()))?;
        for elem in self.rows() {
            set_serializer.serialize_element(&elem)?;
        }
        set_serializer.end()
    }
}

impl<'de, C> Deserialize<'de> for CoordSet<C>
where
    C: Deserialize<'de> + Ord + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CoordSetVisitor<C0> {
            _marker: PhantomData<CoordPair<C0>>,
        }

        impl<'de0, C0> Visitor<'de0> for CoordSetVisitor<C0>
        where
            C0: Deserialize<'de0> + Ord + Clone,
        {
            type Value = CoordSet<C0>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a coordinate set")
            }

            fn visit_seq<A>(
                self,
                mut access: A,
            ) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de0>,
            {
                let mut set = CoordSet::new();
                while let Some(elem) = access.next_element()? {
                    set.insert(elem);
                }
                Ok(set)
            }
        }

        deserializer.deserialize_seq(CoordSetVisitor { _marker: PhantomData })
    }
}

impl<'a, C> IntoIterator for &'a CoordSet<C> {
    type Item = CoordPair<&'a C>;
    type IntoIter = Iter<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows()
    }
}

impl<C> IntoIterator for CoordSet<C>
where
    C: Clone,
{
    type Item = CoordPair<C>;
    type IntoIter = IntoIter<C>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_rows()
    }
}

#[derive(Debug, Clone)]
pub struct Range<'q, 'a, C0, C> {
    inner: map::Range<'q, 'a, C0, C, ()>,
}

impl<'q, 'a, C0, C> Iterator for Range<'q, 'a, C0, C>
where
    C: Ord + Borrow<C0>,
    C0: Ord,
{
    type Item = CoordPair<&'a C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(elem, _)| elem)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'q, 'a, C0, C> DoubleEndedIterator for Range<'q, 'a, C0, C>
where
    C: Ord + Borrow<C0>,
    C0: Ord,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(elem, _)| elem)
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a, C> {
    inner: map::Keys<'a, C, ()>,
}

impl<'a, C> Iterator for Iter<'a, C> {
    type Item = CoordPair<&'a C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, C> DoubleEndedIterator for Iter<'a, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

#[derive(Debug)]
pub struct IntoIter<C> {
    inner: map::IntoKeys<C, ()>,
}

impl<C> Iterator for IntoIter<C>
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

impl<C> DoubleEndedIterator for IntoIter<C>
where
    C: Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}
