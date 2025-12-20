use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, hash_map},
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use crate::{
    entity,
    error::{ErrorCause, OptionExt, Result, ResultMapExt, ResultWrapExt},
    value::TryValue,
};

pub type AnyValue = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawId(u64);

impl RawId {
    pub fn cast_to_index(self) -> usize {
        self.0 as usize
    }

    pub fn typed<C>(self) -> Id<C>
    where
        C: Component,
    {
        Id { inner: self, _marker: PhantomData }
    }
}

impl fmt::Display for RawId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

pub trait Component {
    type Value: TryValue;

    const NAME: &'static str;
}

pub struct Id<C> {
    inner: RawId,
    _marker: PhantomData<C>,
}

impl<C> Id<C> {
    pub fn cast_to_index(self) -> usize {
        self.inner.cast_to_index()
    }

    pub fn raw(self) -> RawId {
        self.inner
    }
}

impl<C> From<Id<C>> for RawId {
    fn from(id: Id<C>) -> Self {
        id.raw()
    }
}

impl<C> fmt::Debug for Id<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedId").field("inner", &self.inner).finish()
    }
}

impl<C> Clone for Id<C> {
    fn clone(&self) -> Self {
        Self { inner: self.inner, _marker: self._marker }
    }
}

impl<C> Copy for Id<C> {}

impl<C> PartialEq for Id<C> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<C> PartialEq<RawId> for Id<C> {
    fn eq(&self, other: &RawId) -> bool {
        self.inner == *other
    }
}

impl<C> PartialEq<Id<C>> for RawId {
    fn eq(&self, other: &Id<C>) -> bool {
        *self == other.inner
    }
}

impl<C> Eq for Id<C> {}

impl<C> PartialOrd for Id<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<C> PartialOrd<RawId> for Id<C> {
    fn partial_cmp(&self, other: &RawId) -> Option<Ordering> {
        self.inner.partial_cmp(other)
    }
}

impl<C> PartialOrd<Id<C>> for RawId {
    fn partial_cmp(&self, other: &Id<C>) -> Option<Ordering> {
        self.partial_cmp(&other.inner)
    }
}

impl<C> Ord for Id<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<C> Hash for Id<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Record {
    id: RawId,
    name: String,
    values: HashMap<entity::Id, AnyValue>,
}

impl Record {
    pub fn new(id: RawId, name: String) -> Self {
        Self { id, name, values: HashMap::new() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[expect(unused)]
    pub fn id(&self) -> RawId {
        self.id
    }

    pub fn get(&self, entity: entity::Id) -> Result<AnyValue> {
        self.values
            .get(&entity)
            .copied()
            .ok_or_ctx(ErrorCause::EntityMissesValue)
            .adding_info("entity.id", entity)
    }

    pub fn set(&mut self, entity: entity::Id, value: AnyValue) -> Result<()> {
        let entry = self
            .values
            .get_mut(&entity)
            .ok_or_ctx(ErrorCause::EntityMissesValue)
            .adding_info("entity.id", entity)?;
        *entry = value;
        Ok(())
    }

    pub fn create_value(&mut self, entity: entity::Id) -> Result<()> {
        match self.values.entry(entity) {
            hash_map::Entry::Vacant(entry) => {
                entry.insert(0);
                Ok(())
            },
            hash_map::Entry::Occupied(_) => {
                Err(ErrorCause::EntityAlreadyHasValue)
                    .wrap_ctx()
                    .adding_info("entity.id", entity)
            },
        }
    }

    pub fn remove_value(&mut self, entity: entity::Id) -> Result<()> {
        self.values
            .remove(&entity)
            .ok_or_ctx(ErrorCause::EntityMissesValue)
            .adding_info("entity.id", entity)?;
        Ok(())
    }

    #[expect(unused)]
    pub fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (entity::Id, AnyValue)> + fmt::Debug + Send + Sync + 'a
    {
        self.values.iter().map(|(id, value)| (*id, *value))
    }

    #[expect(unused)]
    pub fn iter_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = (entity::Id, &'a mut AnyValue)>
    + fmt::Debug
    + Send
    + Sync
    + 'a {
        self.values.iter_mut().map(|(id, value)| (*id, value))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Registry {
    next: RawId,
    records: BTreeMap<RawId, Record>,
    names: HashMap<String, RawId>,
}

impl Registry {
    pub fn new() -> Self {
        Self { next: RawId(0), records: BTreeMap::new(), names: HashMap::new() }
    }

    pub fn create(&mut self, name: String) -> Result<RawId> {
        match self.names.entry(name.clone()) {
            hash_map::Entry::Vacant(entry) => {
                let id = self.next;
                entry.insert(id);
                self.next.0 += 1;
                self.records.insert(id, Record::new(id, name));
                Ok(id)
            },
            hash_map::Entry::Occupied(entry) => {
                Err(ErrorCause::DuplicateComponentName)
                    .wrap_ctx()
                    .adding_info("component.name", name)
                    .adding_info("component.conflict.id", *entry.get())
            },
        }
    }

    pub fn get_or_create(&mut self, name: String) -> RawId {
        match self.names.entry(name.clone()) {
            hash_map::Entry::Vacant(entry) => {
                let id = self.next;
                entry.insert(id);
                self.next.0 += 1;
                self.records.insert(id, Record::new(id, name));
                id
            },
            hash_map::Entry::Occupied(entry) => *entry.get(),
        }
    }

    pub fn id_from_name(&self, name: &str) -> Result<RawId> {
        self.names
            .get(name)
            .copied()
            .ok_or_ctx(ErrorCause::InvalidComponentName)
            .adding_info("component.name", name)
    }

    pub fn get(&self, component: RawId) -> Result<&Record> {
        self.records
            .get(&component)
            .ok_or_ctx(ErrorCause::InvalidComponentId)
            .adding_info("component.id", component)
    }

    pub fn get_mut(&mut self, component: RawId) -> Result<&mut Record> {
        self.records
            .get_mut(&component)
            .ok_or_ctx(ErrorCause::InvalidComponentId)
            .adding_info("component.id", component)
    }

    pub fn create_value(
        &mut self,
        entity: entity::Id,
        component: RawId,
    ) -> Result<()> {
        self.get_mut(component)
            .cause_into()
            .adding_info("entity.id", entity)?
            .create_value(entity)
            .adding_info("component.id", component)
    }

    pub fn remove(&mut self, component: RawId) -> Result<()> {
        self.records
            .remove(&component)
            .ok_or_ctx(ErrorCause::InvalidComponentId)?;
        Ok(())
    }

    pub fn remove_value(
        &mut self,
        entity: entity::Id,
        component: RawId,
    ) -> Result<()> {
        self.get_mut(component)
            .cause_into()
            .adding_info("entity.id", entity)?
            .remove_value(entity)
            .adding_info("component.id", component)
    }

    pub fn remove_values(&mut self, entity: entity::Id) -> Result<()> {
        for record in self.records.values_mut() {
            record.remove_value(entity)?;
        }
        Ok(())
    }

    #[expect(unused)]
    pub fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = &'a Record> + fmt::Debug + Send + Sync + 'a {
        self.records.values()
    }

    #[expect(unused)]
    pub fn iter_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut Record> + fmt::Debug + Send + Sync + 'a
    {
        self.records.values_mut()
    }
}
