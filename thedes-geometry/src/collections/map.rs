use std::{
    collections::{btree_map, BTreeMap},
    mem,
};

use crate::{orientation::Axis, CoordPair};

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
            .expect("The coord map should be mirrored (with_mut yx)");
        let output = modifier(entry_ref);
        let value = entry_ref.clone();
        let entry_ref = self
            .entries
            .x
            .get_mut()
            .get_mut(self.entries.y.key())
            .expect("The coord map should be mirrored (with_mut xy)");
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
