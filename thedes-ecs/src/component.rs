use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, hash_map},
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use thiserror::Error;

use crate::{
    entity,
    error::{CtxResult, OptionExt, ResultMapExt, ResultWrapExt},
    value::TryValue,
};

pub type AnyValue = u64;

#[derive(Debug, Error)]
pub enum CreateError {
    #[error("component is duplicate")]
    Duplicate,
}

#[derive(Debug, Error)]
pub enum GetError {
    #[error("component identifier is invalid")]
    Invalid,
}

#[derive(Debug, Error)]
pub enum GetValueError {
    #[error("entity identifier is invalid")]
    Invalid,
}

#[derive(Debug, Error)]
pub enum SetValueError {
    #[error("entity identifier is invalid")]
    Invalid,
}

#[derive(Debug, Error)]
pub enum CreateValueError {
    #[error("entity already has a value in this component")]
    AlreadyExists,
    #[error("failed to get component")]
    GetComponent(#[from] GetError),
}

#[derive(Debug, Error)]
pub enum RemoveError {
    #[error("component identifier is invalid")]
    Invalid,
}

#[derive(Debug, Error)]
pub enum RemoveValueError {
    #[error("entity identifier is invalid")]
    Invalid,
    #[error("failed to get component")]
    GetComponent(#[from] GetError),
}

#[derive(Debug, Error)]
pub enum IdFromNameError {
    #[error("name is invalid")]
    InvalidName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl Id {
    pub fn cast_to_index(self) -> usize {
        self.0 as usize
    }

    pub fn typed<C>(self) -> TypedId<C>
    where
        C: Component,
    {
        TypedId { inner: self, _marker: PhantomData }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

pub trait Component {
    type Value: TryValue;

    const NAME: &'static str;
}

pub struct TypedId<C> {
    inner: Id,
    _marker: PhantomData<C>,
}

impl<C> TypedId<C> {
    pub fn cast_to_index(self) -> usize {
        self.inner.cast_to_index()
    }

    pub fn raw(self) -> Id {
        self.inner
    }
}

impl<C> From<TypedId<C>> for Id {
    fn from(id: TypedId<C>) -> Self {
        id.raw()
    }
}

impl<C> fmt::Debug for TypedId<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedId").field("inner", &self.inner).finish()
    }
}

impl<C> Clone for TypedId<C> {
    fn clone(&self) -> Self {
        Self { inner: self.inner, _marker: self._marker }
    }
}

impl<C> Copy for TypedId<C> {}

impl<C> PartialEq for TypedId<C> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<C> PartialEq<Id> for TypedId<C> {
    fn eq(&self, other: &Id) -> bool {
        self.inner == *other
    }
}

impl<C> PartialEq<TypedId<C>> for Id {
    fn eq(&self, other: &TypedId<C>) -> bool {
        *self == other.inner
    }
}

impl<C> Eq for TypedId<C> {}

impl<C> PartialOrd for TypedId<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<C> PartialOrd<Id> for TypedId<C> {
    fn partial_cmp(&self, other: &Id) -> Option<Ordering> {
        self.inner.partial_cmp(other)
    }
}

impl<C> PartialOrd<TypedId<C>> for Id {
    fn partial_cmp(&self, other: &TypedId<C>) -> Option<Ordering> {
        self.partial_cmp(&other.inner)
    }
}

impl<C> Ord for TypedId<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<C> Hash for TypedId<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Record {
    id: Id,
    name: String,
    values: HashMap<entity::Id, AnyValue>,
}

impl Record {
    pub fn new(id: Id, name: String) -> Self {
        Self { id, name, values: HashMap::new() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[expect(unused)]
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn get(
        &self,
        entity: entity::Id,
    ) -> CtxResult<AnyValue, GetValueError> {
        self.values
            .get(&entity)
            .copied()
            .ok_or_ctx(GetValueError::Invalid)
            .adding_info("entity.id", entity)
    }

    pub fn set(
        &mut self,
        entity: entity::Id,
        value: AnyValue,
    ) -> CtxResult<(), SetValueError> {
        let entry = self
            .values
            .get_mut(&entity)
            .ok_or_ctx(SetValueError::Invalid)
            .adding_info("entity.id", entity)?;
        *entry = value;
        Ok(())
    }

    pub fn create_value(
        &mut self,
        entity: entity::Id,
    ) -> CtxResult<(), CreateValueError> {
        match self.values.entry(entity) {
            hash_map::Entry::Vacant(entry) => {
                entry.insert(0);
                Ok(())
            },
            hash_map::Entry::Occupied(_) => {
                Err(CreateValueError::AlreadyExists)
                    .wrap_ctx()
                    .adding_info("entity.id", entity)
            },
        }
    }

    pub fn remove_value(
        &mut self,
        entity: entity::Id,
    ) -> CtxResult<(), RemoveValueError> {
        self.values
            .remove(&entity)
            .ok_or_ctx(RemoveValueError::Invalid)
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
    next: Id,
    records: BTreeMap<Id, Record>,
    names: HashMap<String, Id>,
}

impl Registry {
    pub fn new() -> Self {
        Self { next: Id(0), records: BTreeMap::new(), names: HashMap::new() }
    }

    pub fn create(&mut self, name: String) -> CtxResult<Id, CreateError> {
        match self.names.entry(name.clone()) {
            hash_map::Entry::Vacant(entry) => {
                let id = self.next;
                entry.insert(id);
                self.next.0 += 1;
                self.records.insert(id, Record::new(id, name));
                Ok(id)
            },
            hash_map::Entry::Occupied(entry) => Err(CreateError::Duplicate)
                .wrap_ctx()
                .adding_info("component.name", name)
                .adding_info("component.conflict.id", *entry.get()),
        }
    }

    pub fn get_or_create(&mut self, name: String) -> Id {
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

    pub fn id_from_name(&self, name: &str) -> CtxResult<Id, IdFromNameError> {
        self.names
            .get(name)
            .copied()
            .ok_or_ctx(IdFromNameError::InvalidName)
            .adding_info("component.name", name)
    }

    pub fn get(&self, component: Id) -> CtxResult<&Record, GetError> {
        self.records
            .get(&component)
            .ok_or_ctx(GetError::Invalid)
            .adding_info("component.id", component)
    }

    pub fn get_mut(
        &mut self,
        component: Id,
    ) -> CtxResult<&mut Record, GetError> {
        self.records
            .get_mut(&component)
            .ok_or_ctx(GetError::Invalid)
            .adding_info("component.id", component)
    }

    pub fn create_value(
        &mut self,
        entity: entity::Id,
        component: Id,
    ) -> CtxResult<(), CreateValueError> {
        self.get_mut(component)
            .cause_into()
            .adding_info("entity.id", entity)?
            .create_value(entity)
            .adding_info("component.id", component)
    }

    pub fn remove(&mut self, component: Id) -> CtxResult<(), RemoveError> {
        self.records.remove(&component).ok_or_ctx(RemoveError::Invalid)?;
        Ok(())
    }

    pub fn remove_value(
        &mut self,
        entity: entity::Id,
        component: Id,
    ) -> CtxResult<(), RemoveValueError> {
        self.get_mut(component)
            .cause_into()
            .adding_info("entity.id", entity)?
            .remove_value(entity)
            .adding_info("component.id", component)
    }

    pub fn remove_values(
        &mut self,
        entity: entity::Id,
    ) -> CtxResult<(), RemoveValueError> {
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
