use std::{
    collections::{BTreeSet, HashMap},
    fmt,
};

use thiserror::Error;

use crate::{
    component,
    error::{CtxResult, OptionExt, ResultMapExt, ResultWrapExt},
};

#[derive(Debug, Error)]
pub enum GetError {
    #[error("entity identifier is invalid")]
    Invalid,
}

#[derive(Debug, Error)]
pub enum RemoveError {
    #[error("entity identifier is invalid")]
    Invalid,
}

#[derive(Debug, Error)]
pub enum AddComponentError {
    #[error("this component already exists in this entity")]
    AlreadyExists,
}

#[derive(Debug, Error)]
pub enum RemoveComponentError {
    #[error("this component does not exist in this entity")]
    NotInEntity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl Id {
    pub fn cast_to_index(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Record {
    id: Id,
    components: BTreeSet<component::Id>,
}

impl Record {
    pub fn new(id: Id) -> Self {
        Self { id, components: BTreeSet::new() }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    #[expect(unused)]
    pub fn components<'a>(
        &'a self,
    ) -> impl Iterator<Item = component::Id> + fmt::Debug + Send + Sync + 'a
    {
        self.components.iter().copied()
    }

    pub fn has_component(&self, component: component::Id) -> bool {
        self.components.contains(&component)
    }

    pub fn add_component(
        &mut self,
        component: component::Id,
    ) -> CtxResult<(), AddComponentError> {
        if self.components.insert(component) {
            Ok(())
        } else {
            Err(AddComponentError::AlreadyExists)
                .wrap_ctx()
                .adding_info("entity.id", self.id())
                .adding_info("component.id", component)
        }
    }

    pub fn remove_component(
        &mut self,
        component: component::Id,
    ) -> CtxResult<(), RemoveComponentError> {
        if self.components.remove(&component) {
            Ok(())
        } else {
            Err(RemoveComponentError::NotInEntity)
                .wrap_ctx()
                .adding_info("entity.id", self.id())
                .adding_info("component.id", component)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Registry {
    next: Id,
    records: HashMap<Id, Record>,
}

impl Registry {
    pub fn new() -> Self {
        Self { next: Id(0), records: HashMap::new() }
    }

    pub fn create(&mut self) -> Id {
        let id = self.next;
        self.next.0 += 1;
        self.records.insert(id, Record::new(id));
        id
    }

    #[expect(unused)]
    pub fn get(&self, entity: Id) -> CtxResult<&Record, GetError> {
        self.records
            .get(&entity)
            .ok_or_ctx(GetError::Invalid)
            .adding_info("entity.id", entity)
    }

    pub fn get_mut(&mut self, entity: Id) -> CtxResult<&mut Record, GetError> {
        self.records
            .get_mut(&entity)
            .ok_or_ctx(GetError::Invalid)
            .adding_info("entity.id", entity)
    }

    pub fn remove(&mut self, entity: Id) -> CtxResult<(), RemoveError> {
        self.records
            .remove(&entity)
            .ok_or_ctx(RemoveError::Invalid)
            .adding_info("entity.id", entity)?;
        Ok(())
    }

    pub fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = &'a Record> + fmt::Debug + Send + Sync + 'a {
        self.records.values()
    }

    pub fn iter_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut Record> + fmt::Debug + Send + Sync + 'a
    {
        self.records.values_mut()
    }
}
