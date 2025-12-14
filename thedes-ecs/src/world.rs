use thiserror::Error;

use crate::{
    component::{self, AnyValue},
    entity,
    error::{CtxResult, ResultMapExt},
    system::{
        self,
        IntoComponents,
        SystemRunner,
        TypedComponentList,
        TypedEntriesComponents,
        TypedSystemRunner,
    },
    value::{self, FromPrimitiveError, RawEntry, ToPrimitiveError, TryValue},
};

#[derive(Debug, Error)]
pub enum GetValueError {
    #[error("failed to get component")]
    GetComponent(#[from] component::GetError),
    #[error("failed to get component value")]
    GetComponentValue(#[from] component::GetValueError),
    #[error("failed to convert primitive value to higher-level value")]
    FromPrimitive(#[from] value::FromPrimitiveError),
}

#[derive(Debug, Error)]
pub enum SetValueError {
    #[error("failed to get component")]
    GetComponent(#[from] component::GetError),
    #[error("failed to set component value")]
    SetComponentValue(#[from] component::SetValueError),
    #[error("failed to convert higher-level value to primitive value")]
    ToPrimitive(#[from] value::ToPrimitiveError),
}

#[derive(Debug, Error)]
pub enum CreateValueError {
    #[error("failed to create value in component")]
    Component(#[from] component::CreateValueError),
    #[error("failed to get entity")]
    GetEntity(#[from] entity::GetError),
    #[error("failed to add component in entity")]
    AddComponent(#[from] entity::AddComponentError),
}

#[derive(Debug, Error)]
pub enum RemoveValueError {
    #[error("failed to remove value in component")]
    Component(#[from] component::RemoveValueError),
    #[error("failed to get entity")]
    GetEntity(#[from] entity::GetError),
    #[error("failed to remove component in entity")]
    RemoveComponent(#[from] entity::RemoveComponentError),
}

#[derive(Debug, Error)]
pub enum RemoveComponentError {
    #[error("failed to remove component itself")]
    Component(#[from] component::RemoveError),
    #[error("failed to remove component in entity")]
    Entity(#[from] entity::RemoveComponentError),
}

#[derive(Debug, Error)]
pub enum RemoveEntityError {
    #[error("failed to remove entity itself")]
    Entity(#[from] entity::RemoveError),
    #[error("failed to remove entity values")]
    Values(#[from] component::RemoveValueError),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to get component")]
    GetComponent(#[from] component::GetError),
    #[error("failed to get component value")]
    GetComponentValue(#[from] component::GetValueError),
    #[error("failed to set component value")]
    SetComponentValue(#[from] component::SetValueError),
    #[error("failed to get value")]
    GetValue(#[from] GetValueError),
    #[error("failed to set value")]
    SetValue(#[from] SetValueError),
    #[error("failed to remove value")]
    RemoveValue(#[from] RemoveValueError),
    #[error("failed to remove entity")]
    RemoveEntity(#[from] RemoveEntityError),
    #[error("failed to remove entity")]
    RemoveComponent(#[from] RemoveComponentError),
    #[error("failed to run tick")]
    CreateValue(#[from] CreateValueError),
    #[error("failed to convert from primitive")]
    FromPrimitive(#[from] FromPrimitiveError),
    #[error("failed to convert to primitive")]
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
        S: TypedSystemRunner<C>,
        C: IntoComponents + TypedComponentList + TypedEntriesComponents<A>,
    {
        self.systems.create_typed(components, runner)
    }

    pub fn create_value(
        &mut self,
        entity: entity::Id,
        component: impl Into<component::Id>,
    ) -> CtxResult<(), CreateValueError> {
        let component = component.into();
        self.components.create_value(entity, component).cause_into()?;
        self.entities
            .get_mut(entity)
            .cause_into()
            .adding_info("component.id", component)?
            .add_component(component)
            .cause_into()?;
        Ok(())
    }

    pub fn remove_entity(
        &mut self,
        entity: entity::Id,
    ) -> CtxResult<(), RemoveEntityError> {
        self.entities.remove(entity).cause_into()?;
        self.components.remove_values(entity).cause_into()?;
        Ok(())
    }

    pub fn remove_component(
        &mut self,
        component: impl Into<component::Id>,
    ) -> CtxResult<(), RemoveComponentError> {
        let component = component.into();
        self.components.remove(component).cause_into()?;
        for entity in self.entities.iter_mut() {
            entity.remove_component(component).cause_into()?;
        }
        Ok(())
    }

    pub fn remove_system(
        &mut self,
        system: system::Id,
    ) -> CtxResult<(), system::RemoveError> {
        self.systems.remove(system)
    }

    pub fn remove_value(
        &mut self,
        entity: entity::Id,
        component: impl Into<component::Id>,
    ) -> CtxResult<(), RemoveValueError> {
        let component = component.into();
        self.components.remove_value(entity, component).cause_into()?;
        self.entities
            .get_mut(entity)
            .cause_into()
            .adding_info("component.id", component)?
            .remove_component(component)
            .cause_into()?;
        Ok(())
    }

    pub fn get_primitive(
        &self,
        entity: entity::Id,
        component: impl Into<component::Id>,
    ) -> CtxResult<AnyValue, GetValueError> {
        let component = component.into();
        self.components
            .get(component)
            .cause_into()
            .adding_info("entity.id", entity)?
            .get(entity)
            .cause_into()
            .adding_info("component.id", component)
    }

    pub fn set_primitive(
        &mut self,
        entity: entity::Id,
        component: impl Into<component::Id>,
        primitive: AnyValue,
    ) -> CtxResult<(), SetValueError> {
        let component = component.into();
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
        component: impl Into<component::Id>,
    ) -> CtxResult<V, GetValueError>
    where
        V: TryValue,
    {
        let component = component.into();
        let primitive = self.get_primitive(entity, component)?;
        V::try_from_primitive(primitive).cause_into()
    }

    pub fn set_value<V>(
        &mut self,
        entity: entity::Id,
        component: impl Into<component::Id>,
        value: V,
    ) -> CtxResult<(), SetValueError>
    where
        V: TryValue,
    {
        let component = component.into();
        let primitive = value.try_to_primitive().cause_into()?;
        self.set_primitive(entity, component, primitive)
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
