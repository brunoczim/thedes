use std::{
    borrow::Borrow,
    collections::{btree_map, BTreeMap},
    mem,
    ops::Bound,
};

use crate::{
    coords::{CoordPairBounds, CoordRange},
    orientation::Axis,
    CoordPair,
};

#[cfg(test)]
mod test;

#[derive(Debug, Clone)]
pub struct CoordMap<K, V> {
    len: usize,
    inner: CoordPair<BTreeMap<K, BTreeMap<K, V>>>,
}

impl<K, V> Default for CoordMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> CoordMap<K, V> {
    pub fn new() -> Self {
        Self { len: 0, inner: CoordPair::from_axes(|_| BTreeMap::new()) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> CoordMap<K, V>
where
    K: Ord,
{
    pub fn contains_key<Q>(&self, key: CoordPair<&Q>) -> bool
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.get(key).is_some()
    }

    pub fn get<Q>(&self, key: CoordPair<&Q>) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let (_, value) = self.get_key_value(key)?;
        Some(value)
    }

    pub fn get_key_value<Q>(
        &self,
        key: CoordPair<&Q>,
    ) -> Option<(CoordPair<&K>, &V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let (key_y, entry) = self.inner.y.get_key_value(key.y)?;
        let (key_x, value) = entry.get_key_value(key.x)?;
        Some((CoordPair { y: key_y, x: key_x }, value))
    }
}

impl<K, V> CoordMap<K, V>
where
    K: Ord,
    V: Clone,
{
    pub fn with_mut<Q, F, T>(
        &mut self,
        key: CoordPair<&Q>,
        modifier: F,
    ) -> Option<T>
    where
        K: Borrow<Q>,
        Q: Ord,
        F: FnOnce(&mut V) -> T,
    {
        let entry_ref = self.inner.y.get_mut(key.y)?.get_mut(key.x)?;
        let output = modifier(entry_ref);
        let value = entry_ref.clone();
        let entry_ref = self
            .inner
            .x
            .get_mut(key.x)
            .and_then(|entry| entry.get_mut(key.y))
            .expect("The coord map should be mirrored (with_mut xy)");
        *entry_ref = value;
        Some(output)
    }
}

impl<K, V> CoordMap<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    pub fn insert(&mut self, key: CoordPair<K>, value: V) -> Option<V> {
        match self.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(value);
                None
            },
            Entry::Occupied(mut entry) => Some(entry.insert(value)),
        }
    }
}

impl<K, V> CoordMap<K, V>
where
    K: Ord,
{
    pub fn remove_entry<Q>(
        &mut self,
        key: CoordPair<&Q>,
    ) -> Option<(CoordPair<K>, V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let yx_tree = self.inner.y.get_mut(key.y)?;
        let (key_x, value) = yx_tree.remove_entry(key.x)?;
        self.len -= 1;
        if yx_tree.is_empty() {
            self.inner.y.remove(key.y);
        }
        let xy_tree = self.inner.x.get_mut(key.x).expect(
            "The coord map should be mirrored (remove_entry get_mut xy)",
        );
        let (key_y, _) = xy_tree
            .remove_entry(key.y)
            .expect("The coord map should be mirrored (remove_entry xy)");
        if xy_tree.is_empty() {
            self.inner.x.remove(key.x);
        }
        Some((CoordPair { y: key_y, x: key_x }, value))
    }

    pub fn remove<Q>(&mut self, key: CoordPair<&Q>) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let (_, value) = self.remove_entry(key)?;
        Some(value)
    }
}

impl<K, V> CoordMap<K, V>
where
    K: Ord + Clone,
{
    pub fn entry(&mut self, key: CoordPair<K>) -> Entry<K, V> {
        match self
            .inner
            .as_mut()
            .zip2_with(key.clone(), |inner, key| inner.entry(key))
        {
            CoordPair {
                y: btree_map::Entry::Occupied(y_entry),
                x: btree_map::Entry::Occupied(x_entry),
            } => {
                let entries = CoordPair { y: y_entry, x: x_entry };
                Entry::Occupied(OccupiedEntry { len: &mut self.len, entries })
            },

            CoordPair {
                y: btree_map::Entry::Vacant(y_entry),
                x: btree_map::Entry::Vacant(x_entry),
            } => Entry::Vacant(VacantEntry {
                len: &mut self.len,
                inner: VacantEntryInner::BothTopLevel(CoordPair {
                    y: y_entry,
                    x: x_entry,
                }),
            }),

            CoordPair {
                y: btree_map::Entry::Occupied(y_entry),
                x: btree_map::Entry::Vacant(x_entry),
            } => Entry::Vacant(VacantEntry {
                len: &mut self.len,
                inner: VacantEntryInner::OneNested {
                    selected_axis: Axis::Y,
                    selected_key: key.y,
                    selected_nested: y_entry.into_mut().entry(key.x),
                    unselected_top_level: x_entry,
                },
            }),

            CoordPair {
                y: btree_map::Entry::Vacant(y_entry),
                x: btree_map::Entry::Occupied(x_entry),
            } => Entry::Vacant(VacantEntry {
                len: &mut self.len,
                inner: VacantEntryInner::OneNested {
                    selected_axis: Axis::X,
                    selected_key: key.x,
                    selected_nested: x_entry.into_mut().entry(key.y),
                    unselected_top_level: y_entry,
                },
            }),
        }
    }
}

impl<K, V> CoordMap<K, V>
where
    K: Ord,
{
    pub fn range<'q, 'a, R, Q>(
        &'a self,
        higher_axis: Axis,
        ranges: R,
    ) -> Range<'q, 'a, Q, K, V>
    where
        R: Into<CoordPairBounds<&'q Q>>,
        K: Borrow<Q>,
        Q: Ord + 'q,
    {
        let (outer_range, (inner_range_start, inner_range_end)) =
            ranges.into().shift_rev_to(higher_axis).into_order();
        let inner_range = (inner_range_start, inner_range_end);
        let mut outer = self.inner[higher_axis].range(outer_range);
        let inner_front =
            outer.next().map(|(key, tree)| (key, tree.range(inner_range)));
        let inner_back =
            outer.next_back().map(|(key, tree)| (key, tree.range(inner_range)));
        Range {
            higher_axis,
            inner_range_start,
            inner_range_end,
            outer,
            inner_front,
            inner_back,
        }
    }

    pub fn next_neighbor<Q>(
        &self,
        axis: Axis,
        key: CoordPair<&Q>,
    ) -> Option<(CoordPair<&K>, &V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let (first_key, second_key) = key.shift_rev_to(axis).into_order();
        let range = CoordRange::with_order(
            (Bound::Excluded(first_key), Bound::Unbounded),
            second_key ..= second_key,
        );
        let mut iter = self.range(axis, range.as_bounds().shift_to(axis));
        iter.next()
    }

    pub fn last_neighbor<Q>(
        &self,
        axis: Axis,
        key: CoordPair<&Q>,
    ) -> Option<(CoordPair<&K>, &V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let (first_key, second_key) = key.shift_rev_to(axis).into_order();
        let range = CoordRange::with_order(
            (Bound::Excluded(first_key), Bound::Unbounded),
            second_key ..= second_key,
        );
        let mut iter = self.range(axis, range.as_bounds().shift_to(axis));
        iter.next_back()
    }

    pub fn prev_neighbor<Q>(
        &self,
        axis: Axis,
        key: CoordPair<&Q>,
    ) -> Option<(CoordPair<&K>, &V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let (first_key, second_key) = key.shift_rev_to(axis).into_order();
        let range =
            CoordRange::with_order(.. first_key, second_key ..= second_key);
        let mut iter = self.range(axis, range.as_bounds().shift_to(axis));
        iter.next_back()
    }

    pub fn first_neighbor<Q>(
        &self,
        axis: Axis,
        key: CoordPair<&Q>,
    ) -> Option<(CoordPair<&K>, &V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let (first_key, second_key) = key.shift_rev_to(axis).into_order();
        let range =
            CoordRange::with_order(.. first_key, second_key ..= second_key);
        let mut iter = self.range(axis, range.as_bounds().shift_to(axis));
        iter.next()
    }
}

impl<K, V> CoordMap<K, V> {
    pub fn iter(&self, higher_axis: Axis) -> Iter<K, V> {
        let mut outer = self.inner[higher_axis].iter();
        let inner_front = outer.next().map(|(key, tree)| (key, tree.iter()));
        let inner_back =
            outer.next_back().map(|(key, tree)| (key, tree.iter()));
        Iter { axis: higher_axis, outer, inner_front, inner_back }
    }

    pub fn rows(&self) -> Iter<K, V> {
        self.iter(Axis::Y)
    }

    pub fn columns(&self) -> Iter<K, V> {
        self.iter(Axis::X)
    }

    pub fn keys(&self, higher_axis: Axis) -> Keys<K, V> {
        Keys { inner: self.iter(higher_axis) }
    }

    pub fn key_rows(&self) -> Keys<K, V> {
        self.keys(Axis::Y)
    }

    pub fn key_columns(&self) -> Keys<K, V> {
        self.keys(Axis::X)
    }

    pub fn values(&self, higher_axis: Axis) -> Values<K, V> {
        Values { inner: self.iter(higher_axis) }
    }

    pub fn value_rows(&self) -> Values<K, V> {
        self.values(Axis::Y)
    }

    pub fn value_columns(&self) -> Values<K, V> {
        self.values(Axis::X)
    }
}

impl<K, V> CoordMap<K, V>
where
    K: Clone,
{
    pub fn into_iter_with(self, higher_axis: Axis) -> IntoIter<K, V> {
        let mut outer = self.inner.extract(higher_axis).into_iter();
        let inner_front =
            outer.next().map(|(key, tree)| (key, tree.into_iter()));
        let inner_back =
            outer.next_back().map(|(key, tree)| (key, tree.into_iter()));
        IntoIter { axis: higher_axis, outer, inner_front, inner_back }
    }

    pub fn into_rows(self) -> IntoIter<K, V> {
        self.into_iter_with(Axis::Y)
    }

    pub fn into_columns(self) -> IntoIter<K, V> {
        self.into_iter_with(Axis::X)
    }

    pub fn into_keys(self, higher_axis: Axis) -> IntoKeys<K, V> {
        IntoKeys { inner: self.into_iter_with(higher_axis) }
    }

    pub fn into_key_rows(self) -> IntoKeys<K, V> {
        self.into_keys(Axis::Y)
    }

    pub fn into_key_columns(self) -> IntoKeys<K, V> {
        self.into_keys(Axis::X)
    }
}

impl<K, V> CoordMap<K, V> {
    pub fn into_values(self, higher_axis: Axis) -> IntoValues<K, V> {
        let mut outer = self.inner.extract(higher_axis).into_values();
        let inner_front = outer.next().map(BTreeMap::into_values);
        let inner_back = outer.next_back().map(BTreeMap::into_values);
        IntoValues { outer, inner_front, inner_back }
    }

    pub fn into_value_rows(self) -> IntoValues<K, V> {
        self.into_values(Axis::Y)
    }

    pub fn into_value_columns(self) -> IntoValues<K, V> {
        self.into_values(Axis::X)
    }
}

impl<'a, K, V> IntoIterator for &'a CoordMap<K, V> {
    type Item = (CoordPair<&'a K>, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows()
    }
}

impl<K, V> IntoIterator for CoordMap<K, V>
where
    K: Clone,
{
    type Item = (CoordPair<K>, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_rows()
    }
}

enum VacantEntryInner<'a, K, V> {
    BothTopLevel(CoordPair<btree_map::VacantEntry<'a, K, BTreeMap<K, V>>>),
    OneNested {
        selected_axis: Axis,
        selected_key: K,
        selected_nested: btree_map::Entry<'a, K, V>,
        unselected_top_level: btree_map::VacantEntry<'a, K, BTreeMap<K, V>>,
    },
}

impl<'a, K, V> VacantEntryInner<'a, K, V>
where
    K: Ord,
{
    pub fn key(&self) -> CoordPair<&K> {
        match self {
            Self::BothTopLevel(entries) => {
                entries.as_ref().map(btree_map::VacantEntry::key)
            },

            Self::OneNested {
                selected_key,
                unselected_top_level,
                selected_axis,
                ..
            } => {
                CoordPair::with_order(selected_key, unselected_top_level.key())
                    .shift_to(*selected_axis)
            },
        }
    }

    pub fn into_key(self) -> CoordPair<K> {
        match self {
            Self::BothTopLevel(entries) => {
                entries.map(btree_map::VacantEntry::into_key)
            },

            Self::OneNested {
                selected_axis,
                selected_key,
                unselected_top_level,
                ..
            } => CoordPair::with_order(
                selected_key,
                unselected_top_level.into_key(),
            )
            .shift_to(selected_axis),
        }
    }

    pub fn insert(self, value: V) -> &'a V
    where
        K: Clone,
        V: Clone,
    {
        match self {
            Self::BothTopLevel(entries) => {
                let key = CoordPair {
                    y: entries.y.key().clone(),
                    x: entries.x.key().clone(),
                };
                entries
                    .y
                    .insert(BTreeMap::new())
                    .entry(key.x)
                    .or_insert(value.clone());
                let entry_ref = entries
                    .x
                    .insert(BTreeMap::new())
                    .entry(key.y)
                    .or_insert(value);
                &*entry_ref
            },
            Self::OneNested {
                selected_key,
                selected_nested,
                unselected_top_level,
                ..
            } => {
                match selected_nested {
                    btree_map::Entry::Vacant(entry) => {
                        entry.insert(value.clone());
                    },
                    btree_map::Entry::Occupied(mut entry) => {
                        entry.insert(value.clone());
                    },
                }
                let entry_ref = unselected_top_level
                    .insert(BTreeMap::new())
                    .entry(selected_key)
                    .or_insert(value);
                &*entry_ref
            },
        }
    }
}

pub struct VacantEntry<'a, K, V> {
    len: &'a mut usize,
    inner: VacantEntryInner<'a, K, V>,
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Ord,
{
    pub fn key(&self) -> CoordPair<&K> {
        self.inner.key()
    }

    pub fn into_key(self) -> CoordPair<K> {
        self.inner.into_key()
    }

    pub fn insert(self, value: V) -> &'a V
    where
        V: Clone,
        K: Clone,
    {
        *self.len += 1;
        self.inner.insert(value)
    }
}

pub struct OccupiedEntry<'a, K, V> {
    len: &'a mut usize,
    entries: CoordPair<btree_map::OccupiedEntry<'a, K, BTreeMap<K, V>>>,
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Ord,
{
    pub fn key(&self) -> CoordPair<&K> {
        self.entries.as_ref().map(btree_map::OccupiedEntry::key)
    }

    pub fn remove_entry(mut self) -> (CoordPair<K>, V)
    where
        K: Clone,
    {
        *self.len -= 1;
        let value = self.entries.y.get_mut().remove(self.entries.x.key());
        self.entries.x.get_mut().remove(self.entries.y.key());
        let keys = self.entries.map(|entry| {
            if entry.get().is_empty() {
                let (key, _) = entry.remove_entry();
                key
            } else {
                entry.key().clone()
            }
        });
        (keys, value.expect("The coord map should be mirrored (remove entry)"))
    }

    pub fn get(&self) -> &V {
        self.entries
            .y
            .get()
            .get(self.entries.x.key())
            .expect("The coord map should be mirrored (get entry value)")
    }

    pub fn with_mut<F, T>(&mut self, modifier: F) -> T
    where
        V: Clone,
        F: FnOnce(&mut V) -> T,
    {
        let entry_ref = self
            .entries
            .y
            .get_mut()
            .get_mut(self.entries.x.key())
            .expect("The coord map should be mirrored (with_mut entry yx)");
        let output = modifier(entry_ref);
        let value = entry_ref.clone();
        let entry_ref = self
            .entries
            .x
            .get_mut()
            .get_mut(self.entries.y.key())
            .expect("The coord map should be mirrored (with_mut entry xy)");
        *entry_ref = value;
        output
    }

    pub fn into_ref(self) -> &'a V {
        self.entries
            .y
            .into_mut()
            .get(self.entries.x.key())
            .expect("The coord map shpuçd be mirrored (into_ref)")
    }

    pub fn insert(&mut self, value: V) -> V
    where
        V: Clone,
    {
        let entry_ref =
            self.entries.y.get_mut().get_mut(self.entries.x.key()).expect(
                "The coord map should be mirrored (insert entry value yx)",
            );
        let old_value = mem::replace(entry_ref, value.clone());

        let entry_ref =
            self.entries.x.get_mut().get_mut(self.entries.y.key()).expect(
                "The coord map should be mirrored (insert entry value xy)",
            );
        *entry_ref = value;

        old_value
    }

    pub fn remove(self) -> V
    where
        K: Clone,
    {
        let (_, value) = self.remove_entry();
        value
    }
}

pub enum Entry<'a, K, V> {
    Vacant(VacantEntry<'a, K, V>),
    Occupied(OccupiedEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Ord,
{
    pub fn or_insert(self, default: V) -> &'a V
    where
        K: Clone,
        V: Clone,
    {
        match self {
            Self::Vacant(entry) => entry.insert(default),
            Self::Occupied(entry) => entry.into_ref(),
        }
    }

    pub fn or_insert_with<F>(self, default: F) -> &'a V
    where
        K: Clone,
        V: Clone,
        F: FnOnce() -> V,
    {
        match self {
            Self::Vacant(entry) => entry.insert(default()),
            Self::Occupied(entry) => entry.into_ref(),
        }
    }

    pub fn or_insert_with_key<F>(self, default: F) -> &'a V
    where
        K: Clone,
        V: Clone,
        F: FnOnce(CoordPair<&K>) -> V,
    {
        match self {
            Self::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            },
            Self::Occupied(entry) => entry.into_ref(),
        }
    }

    pub fn key(&self) -> CoordPair<&K> {
        match self {
            Self::Vacant(entry) => entry.key(),
            Self::Occupied(entry) => entry.key(),
        }
    }

    pub fn and_modify<F>(mut self, modifier: F) -> Self
    where
        K: Clone,
        V: Clone,
        F: FnOnce(&mut V),
    {
        match &mut self {
            Self::Occupied(entry) => {
                entry.with_mut(modifier);
            },
            _ => (),
        }
        self
    }

    pub fn or_default(self) -> &'a V
    where
        K: Clone,
        V: Clone + Default,
    {
        self.or_insert_with(Default::default)
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a, K, V> {
    axis: Axis,
    outer: btree_map::Iter<'a, K, BTreeMap<K, V>>,
    inner_front: Option<(&'a K, btree_map::Iter<'a, K, V>)>,
    inner_back: Option<(&'a K, btree_map::Iter<'a, K, V>)>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (CoordPair<&'a K>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (outer_key, mut inner_iter) =
                self.inner_front.take().or_else(|| self.inner_back.take())?;
            match inner_iter.next() {
                Some((inner_key, value)) => {
                    self.inner_front = Some((outer_key, inner_iter));
                    let key = CoordPair::with_order(outer_key, inner_key)
                        .shift_to(self.axis);
                    break Some((key, value));
                },
                None => {
                    self.inner_front =
                        self.outer.next().map(|(key, tree)| (key, tree.iter()));
                },
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (outer_low, _) = self.outer.size_hint();
        let front_low = self
            .inner_front
            .as_ref()
            .map(|(_, iter)| iter.size_hint())
            .map(|(low, _)| low)
            .unwrap_or_default();
        let back_low = self
            .inner_back
            .as_ref()
            .map(|(_, iter)| iter.size_hint())
            .map(|(low, _)| low)
            .unwrap_or_default();
        (outer_low + front_low + back_low, None)
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let (outer_key, mut inner_iter) =
                self.inner_back.take().or_else(|| self.inner_front.take())?;
            match inner_iter.next_back() {
                Some((inner_key, value)) => {
                    self.inner_back = Some((outer_key, inner_iter));
                    let key = CoordPair::with_order(outer_key, inner_key)
                        .shift_to(self.axis);
                    break Some((key, value));
                },
                None => {
                    self.inner_back = self
                        .outer
                        .next_back()
                        .map(|(key, tree)| (key, tree.iter()));
                },
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Keys<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = CoordPair<&'a K>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, _)| key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(key, _)| key)
    }
}

#[derive(Debug, Clone)]
pub struct Values<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(_, value)| value)
    }
}

#[derive(Debug)]
pub struct IntoIter<K, V> {
    axis: Axis,
    outer: btree_map::IntoIter<K, BTreeMap<K, V>>,
    inner_front: Option<(K, btree_map::IntoIter<K, V>)>,
    inner_back: Option<(K, btree_map::IntoIter<K, V>)>,
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Clone,
{
    type Item = (CoordPair<K>, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (outer_key, mut inner_iter) =
                self.inner_front.take().or_else(|| self.inner_back.take())?;
            match inner_iter.next() {
                Some((inner_key, value)) => {
                    self.inner_front = Some((outer_key.clone(), inner_iter));
                    let key = CoordPair::with_order(outer_key, inner_key)
                        .shift_to(self.axis);
                    break Some((key, value));
                },
                None => {
                    self.inner_front = self
                        .outer
                        .next()
                        .map(|(key, tree)| (key, tree.into_iter()));
                },
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (outer_low, _) = self.outer.size_hint();
        let front_low = self
            .inner_front
            .as_ref()
            .map(|(_, iter)| iter.size_hint())
            .map(|(low, _)| low)
            .unwrap_or_default();
        let back_low = self
            .inner_back
            .as_ref()
            .map(|(_, iter)| iter.size_hint())
            .map(|(low, _)| low)
            .unwrap_or_default();
        (outer_low + front_low + back_low, None)
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V>
where
    K: Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let (outer_key, mut inner_iter) =
                self.inner_back.take().or_else(|| self.inner_front.take())?;
            match inner_iter.next_back() {
                Some((inner_key, value)) => {
                    self.inner_back = Some((outer_key.clone(), inner_iter));
                    let key = CoordPair::with_order(outer_key, inner_key)
                        .shift_to(self.axis);
                    break Some((key, value));
                },
                None => {
                    self.inner_back = self
                        .outer
                        .next_back()
                        .map(|(key, tree)| (key, tree.into_iter()));
                },
            }
        }
    }
}

#[derive(Debug)]
pub struct IntoKeys<K, V> {
    inner: IntoIter<K, V>,
}

impl<K, V> Iterator for IntoKeys<K, V>
where
    K: Clone,
{
    type Item = CoordPair<K>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, _)| key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V>
where
    K: Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(key, _)| key)
    }
}

#[derive(Debug)]
pub struct IntoValues<K, V> {
    outer: btree_map::IntoValues<K, BTreeMap<K, V>>,
    inner_front: Option<btree_map::IntoValues<K, V>>,
    inner_back: Option<btree_map::IntoValues<K, V>>,
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut inner_iter =
                self.inner_front.take().or_else(|| self.inner_back.take())?;
            match inner_iter.next() {
                Some(value) => {
                    self.inner_front = Some(inner_iter);
                    break Some(value);
                },
                None => {
                    self.inner_front =
                        self.outer.next().map(BTreeMap::into_values)
                },
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (outer_low, _) = self.outer.size_hint();
        let front_low = self
            .inner_front
            .as_ref()
            .map(|iter| iter.size_hint())
            .map(|(low, _)| low)
            .unwrap_or_default();
        let back_low = self
            .inner_back
            .as_ref()
            .map(|iter| iter.size_hint())
            .map(|(low, _)| low)
            .unwrap_or_default();
        (outer_low + front_low + back_low, None)
    }
}

impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let mut inner_iter =
                self.inner_back.take().or_else(|| self.inner_front.take())?;
            match inner_iter.next_back() {
                Some(value) => {
                    self.inner_back = Some(inner_iter);
                    break Some(value);
                },
                None => {
                    self.inner_back =
                        self.outer.next_back().map(BTreeMap::into_values)
                },
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Range<'q, 'a, Q, K, V> {
    higher_axis: Axis,
    inner_range_start: Bound<&'q Q>,
    inner_range_end: Bound<&'q Q>,
    outer: btree_map::Range<'a, K, BTreeMap<K, V>>,
    inner_front: Option<(&'a K, btree_map::Range<'a, K, V>)>,
    inner_back: Option<(&'a K, btree_map::Range<'a, K, V>)>,
}

impl<'q, 'a, Q, K, V> Iterator for Range<'q, 'a, Q, K, V>
where
    K: Ord + Borrow<Q>,
    Q: Ord,
{
    type Item = (CoordPair<&'a K>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (outer_key, mut inner_iter) =
                self.inner_front.take().or_else(|| self.inner_back.take())?;
            match inner_iter.next() {
                Some((inner_key, value)) => {
                    self.inner_front = Some((outer_key, inner_iter));
                    let key = CoordPair::with_order(outer_key, inner_key)
                        .shift_to(self.higher_axis);
                    break Some((key, value));
                },
                None => {
                    let inner_range =
                        (self.inner_range_start, self.inner_range_end);
                    self.inner_front = self
                        .outer
                        .next()
                        .map(|(key, tree)| (key, tree.range(inner_range)));
                },
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (outer_low, _) = self.outer.size_hint();
        let front_low = self
            .inner_front
            .as_ref()
            .map(|(_, iter)| iter.size_hint())
            .map(|(low, _)| low)
            .unwrap_or_default();
        let back_low = self
            .inner_back
            .as_ref()
            .map(|(_, iter)| iter.size_hint())
            .map(|(low, _)| low)
            .unwrap_or_default();
        (outer_low + front_low + back_low, None)
    }
}

impl<'q, 'a, Q, K, V> DoubleEndedIterator for Range<'q, 'a, Q, K, V>
where
    K: Ord + Borrow<Q>,
    Q: Ord,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let (outer_key, mut inner_iter) =
                self.inner_back.take().or_else(|| self.inner_front.take())?;
            match inner_iter.next_back() {
                Some((inner_key, value)) => {
                    self.inner_back = Some((outer_key, inner_iter));
                    let key = CoordPair::with_order(outer_key, inner_key)
                        .shift_to(self.higher_axis);
                    break Some((key, value));
                },
                None => {
                    let inner_range =
                        (self.inner_range_start, self.inner_range_end);
                    self.inner_back = self
                        .outer
                        .next_back()
                        .map(|(key, tree)| (key, tree.range(inner_range)));
                },
            }
        }
    }
}
