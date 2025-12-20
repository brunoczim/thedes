use std::{
    collections::{BTreeSet, HashMap, hash_map},
    fmt,
};

use crate::{
    component::{self, AnyValue},
    error::{ErrorCause, OptionExt, Result, ResultMapExt, ResultWrapExt},
    value::Value,
};

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

impl Value for Id {
    fn from_primitive(primitive: AnyValue) -> Self {
        Self(primitive)
    }

    fn to_primitive(&self) -> AnyValue {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Record {
    id: Id,
    components: BTreeSet<component::RawId>,
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
    ) -> impl Iterator<Item = component::RawId> + fmt::Debug + Send + Sync + 'a
    {
        self.components.iter().copied()
    }

    pub fn has_component(&self, component: component::RawId) -> bool {
        self.components.contains(&component)
    }

    pub fn add_component(&mut self, component: component::RawId) -> Result<()> {
        if self.components.insert(component) {
            Ok(())
        } else {
            Err(ErrorCause::EntityAlreadyHasComponent)
                .wrap_ctx()
                .adding_info("entity.id", self.id())
                .adding_info("component.id", component)
        }
    }

    pub fn remove_component(
        &mut self,
        component: component::RawId,
    ) -> Result<()> {
        if self.components.remove(&component) {
            Ok(())
        } else {
            Err(ErrorCause::ComponentNotInEntity)
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
    names: HashMap<String, Id>,
}

impl Registry {
    pub fn new() -> Self {
        Self { next: Id(0), records: HashMap::new(), names: HashMap::new() }
    }

    pub fn create(&mut self) -> Id {
        let id = self.next;
        self.next.0 += 1;
        self.records.insert(id, Record::new(id));
        id
    }

    pub fn get_or_create(&mut self, name: String) -> Id {
        match self.names.entry(name.clone()) {
            hash_map::Entry::Vacant(entry) => {
                let id = self.next;
                entry.insert(id);
                self.next.0 += 1;
                self.records.insert(id, Record::new(id));
                id
            },
            hash_map::Entry::Occupied(entry) => *entry.get(),
        }
    }

    pub fn add_name(&mut self, entity: Id, name: String) -> Result<()> {
        match self.names.entry(name.clone()) {
            hash_map::Entry::Vacant(entry) => {
                if !self.records.contains_key(&entity) {
                    Err(ErrorCause::InvalidEntityId)
                        .wrap_ctx()
                        .adding_info("entity.id", entity)
                        .adding_info("entity.name", &name)?;
                }
                entry.insert(entity);
            },
            hash_map::Entry::Occupied(entry) => {
                if *entry.get() != entity {
                    Err(ErrorCause::EntityNameTaken)
                        .wrap_ctx()
                        .adding_info("entity.id", entity)
                        .adding_info("entity.conflict.id", *entry.get())
                        .adding_info("entity.name", &name)?
                }
            },
        }
        Ok(())
    }

    pub fn remove_name(&mut self, name: &str) -> Result<Id> {
        self.names
            .remove(name)
            .ok_or_ctx(ErrorCause::InvalidEntityName)
            .adding_info("entity.name", name)
    }

    pub fn id_from_name(&self, name: &str) -> Result<Id> {
        self.names
            .get(name)
            .copied()
            .ok_or_ctx(ErrorCause::EntityNameTaken)
            .adding_info("entity.name", name)
    }

    #[expect(unused)]
    pub fn get(&self, entity: Id) -> Result<&Record> {
        self.records
            .get(&entity)
            .ok_or_ctx(ErrorCause::InvalidEntityId)
            .adding_info("entity.id", entity)
    }

    pub fn get_mut(&mut self, entity: Id) -> Result<&mut Record> {
        self.records
            .get_mut(&entity)
            .ok_or_ctx(ErrorCause::InvalidEntityId)
            .adding_info("entity.id", entity)
    }

    pub fn remove(&mut self, entity: Id) -> Result<()> {
        self.records
            .remove(&entity)
            .ok_or_ctx(ErrorCause::InvalidEntityId)
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
