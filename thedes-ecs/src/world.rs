use thiserror::Error;

use crate::{
    component::{
        self,
        AnyValue,
        CreateValueError,
        GetValueError,
        RemoveValueError,
        SetValueError,
    },
    entity::{self, AddComponentError, RemoveComponentError},
    error::{CtxResult, ResultMapExt},
    system::{
        self,
        IntoComponents,
        SystemRunner,
        TypedComponentList,
        TypedEntriesComponents,
    },
    value::{FromPrimitiveError, RawEntry, ToPrimitiveError, TryValue},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to get entity")]
    GetEntity(#[from] entity::GetError),
    #[error("failed to remove entity")]
    RemoveEntity(#[from] entity::RemoveError),
    #[error("failed to get component")]
    GetComponent(#[from] component::GetError),
    #[error("failed to remove component")]
    RemoveComponent(#[from] component::RemoveError),
    #[error("failed to add a component")]
    AddEntityComponent(#[from] AddComponentError),
    #[error("failed to remove a component")]
    RemoveEntityComponent(#[from] RemoveComponentError),
    #[error("failed to create value")]
    CreateValue(#[from] CreateValueError),
    #[error("failed to get a value")]
    GetValue(#[from] GetValueError),
    #[error("failed to set a value")]
    SetValue(#[from] SetValueError),
    #[error("failed to remove a value")]
    RemoveValueError(#[from] RemoveValueError),
    #[error("failed to remove a system")]
    RemoveSystem(#[from] system::RemoveError),
    #[error("failed to convert value from primitive")]
    FromPrimitive(#[from] FromPrimitiveError),
    #[error("failed to convert value to primitive")]
    ToPrimitive(#[from] ToPrimitiveError),
    #[error("missing entry in system call")]
    MissingEntry,
}

#[derive(Debug, Clone)]
pub struct World {
    components: component::Registry,
    entities: entity::Registry,
    systems: system::Registry,
}

impl World {
    pub fn new() -> Self {
        Self {
            components: component::Registry::new(),
            entities: entity::Registry::new(),
            systems: system::Registry::new(),
        }
    }

    pub fn create_entity(&mut self) -> entity::Id {
        self.entities.create()
    }

    pub fn create_component_raw(&mut self) -> component::Id {
        self.components.create()
    }

    pub fn create_component<V>(&mut self) -> component::TypedId<V>
    where
        V: TryValue,
    {
        self.components.create().typed()
    }

    pub fn create_system_raw<S>(
        &mut self,
        components: impl IntoIterator<Item = impl Into<component::Id>>,
        runner: S,
    ) -> system::Id
    where
        S: SystemRunner,
    {
        self.systems.create_raw(components.into_iter().map(Into::into), runner)
    }

    pub fn create_system<S, A, C>(
        &mut self,
        components: C,
        runner: S,
    ) -> system::Id
    where
        C: IntoComponents + TypedComponentList + TypedEntriesComponents<A>,
        S: for<'b> FnMut(C::Entries<'b>) -> CtxResult<(), Error>,
        S: Clone + Send + Sync + 'static,
    {
        self.systems.create_typed(components, runner)
    }

    pub fn create_value_raw(
        &mut self,
        entity: entity::Id,
        component: component::Id,
    ) -> CtxResult<(), Error> {
        self.components.create_value(entity, component).cause_into()?;
        self.entities
            .get_mut(entity)
            .cause_into()
            .adding_info("component.id", component)?
            .add_component(component)
            .cause_into()?;
        Ok(())
    }

    pub fn create_value<V>(
        &mut self,
        entity: entity::Id,
        component: component::TypedId<V>,
        value: V,
    ) -> CtxResult<(), Error>
    where
        V: TryValue,
    {
        self.create_value_raw(entity, component.raw())?;
        self.set_value(entity, component, value)?;
        Ok(())
    }

    pub fn remove_entity(
        &mut self,
        entity: entity::Id,
    ) -> CtxResult<(), Error> {
        self.entities.remove(entity).cause_into()?;
        self.components.remove_values(entity).cause_into()?;
        Ok(())
    }

    pub fn remove_component(
        &mut self,
        component: component::Id,
    ) -> CtxResult<(), Error> {
        self.components.remove(component).cause_into()?;
        for entity in self.entities.iter_mut() {
            entity.remove_component(component).cause_into()?;
        }
        Ok(())
    }

    pub fn remove_system(
        &mut self,
        system: system::Id,
    ) -> CtxResult<(), Error> {
        self.systems.remove(system).cause_into()?;
        Ok(())
    }

    pub fn remove_value(
        &mut self,
        entity: entity::Id,
        component: component::Id,
    ) -> CtxResult<(), Error> {
        self.components.remove_value(entity, component).cause_into()?;
        self.entities
            .get_mut(entity)
            .cause_into()
            .adding_info("component.id", component)?
            .remove_component(component)
            .cause_into()?;
        Ok(())
    }

    pub fn get_value_raw(
        &self,
        entity: entity::Id,
        component: component::Id,
    ) -> CtxResult<AnyValue, Error> {
        self.components
            .get(component)
            .cause_into()
            .adding_info("entity.id", entity)?
            .get(entity)
            .cause_into()
            .adding_info("component.id", component)
    }

    pub fn set_value_raw(
        &mut self,
        entity: entity::Id,
        component: component::Id,
        primitive: AnyValue,
    ) -> CtxResult<(), Error> {
        self.components
            .get_mut(component)
            .cause_into()
            .adding_info("entity.id", entity)?
            .set(entity, primitive)
            .cause_into()
            .adding_info("component.id", component)
    }

    pub fn get_value<V>(
        &self,
        entity: entity::Id,
        component: component::TypedId<V>,
    ) -> CtxResult<V, Error>
    where
        V: TryValue,
    {
        let primitive = self.get_value_raw(entity, component.raw())?;
        V::try_from_primitive(primitive).cause_into()
    }

    pub fn set_value<V>(
        &mut self,
        entity: entity::Id,
        component: component::TypedId<V>,
        value: V,
    ) -> CtxResult<(), Error>
    where
        V: TryValue,
    {
        let primitive = value.try_to_primitive().cause_into()?;
        self.set_value_raw(entity, component.raw(), primitive)
    }

    pub fn tick(&mut self) -> CtxResult<(), Error> {
        let mut entries = Vec::new();
        for system in self.systems.iter_mut() {
            for entity in self.entities.iter() {
                if system
                    .components()
                    .iter()
                    .all(|component| entity.has_component(*component))
                {
                    for &component in system.components() {
                        let value = self
                            .components
                            .get(component)
                            .cause_into()
                            .adding_info("system.id", system.id())
                            .adding_info("entity.id", entity.id())?
                            .get(entity.id())
                            .cause_into()
                            .adding_info("system.id", system.id())
                            .adding_info("component.id", component)?;
                        let entry = RawEntry::new(value);
                        entries.push(entry);
                    }
                    system.runner().run(&mut entries)?;
                    for (&component, entry) in
                        system.components().iter().zip(&entries)
                    {
                        self.components
                            .get_mut(component)
                            .cause_into()
                            .adding_info("system.id", system.id())
                            .adding_info("entity.id", entity.id())?
                            .set(entity.id(), entry.get_primitive())
                            .cause_into()
                            .adding_info("system.id", system.id())
                            .adding_info("component.id", component)?;
                    }
                    entries.clear();
                }
            }
        }
        Ok(())
    }
}
