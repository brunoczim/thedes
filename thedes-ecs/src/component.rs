use std::{
    collections::{BTreeMap, HashMap, hash_map},
    fmt,
};

use thiserror::Error;

use crate::{
    entity,
    error::{CtxResult, OptionExt, ResultMapExt, ResultWrapExt},
};

pub type AnyValue = u64;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Record {
    id: Id,
    values: HashMap<entity::Id, AnyValue>,
}

impl Record {
    pub fn new(id: Id) -> Self {
        Self { id, values: HashMap::new() }
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
}

impl Registry {
    pub fn new() -> Self {
        Self { next: Id(0), records: BTreeMap::new() }
    }

    pub fn create(&mut self) -> Id {
        let id = self.next;
        self.next.0 += 1;
        self.records.insert(id, Record::new(id));
        id
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
