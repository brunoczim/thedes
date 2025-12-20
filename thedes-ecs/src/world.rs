use crate::{
    component::{self, AnyValue, Component},
    entity::{self},
    error::{Result, ResultMapExt},
    system::{
        self,
        IntoComponents,
        SystemRunner,
        TypedComponentList,
        TypedEntriesComponents,
    },
    value::{RawEntry, TryValue},
};

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

    pub fn get_or_create_entity(
        &mut self,
        name: impl Into<String>,
    ) -> entity::Id {
        self.entities.get_or_create(name.into())
    }

    pub fn add_entity_name(
        &mut self,
        entity: entity::Id,
        name: impl Into<String>,
    ) -> Result<()> {
        self.entities.add_name(entity, name.into())
    }

    pub fn remove_entity_name(&mut self, name: &str) -> Result<entity::Id> {
        self.entities.remove_name(name)
    }

    pub fn get_entity(&self, name: &str) -> Result<entity::Id> {
        self.entities.id_from_name(name)
    }

    pub fn create_component_raw(
        &mut self,
        name: impl Into<String>,
    ) -> Result<component::RawId> {
        self.components.create(name.into())
    }

    pub fn create_component<C>(
        &mut self,
        _component: C,
    ) -> Result<component::Id<C>>
    where
        C: Component,
    {
        Ok(self.create_component_raw(C::NAME)?.typed())
    }

    pub fn get_or_create_component_raw(
        &mut self,
        name: impl Into<String>,
    ) -> component::RawId {
        self.components.get_or_create(name.into())
    }

    pub fn get_or_create_component<C>(
        &mut self,
        _component: C,
    ) -> component::Id<C>
    where
        C: Component,
    {
        self.get_or_create_component_raw(C::NAME).typed()
    }

    pub fn get_component_raw(&self, name: &str) -> Result<component::RawId> {
        self.components.id_from_name(name)
    }

    pub fn get_component_name(
        &self,
        component: impl Into<component::RawId>,
    ) -> Result<&str> {
        Ok(self.components.get(component.into())?.name())
    }

    pub fn get_component<C>(&self, _component: C) -> Result<component::Id<C>>
    where
        C: Component,
    {
        Ok(self.get_component_raw(C::NAME)?.typed())
    }

    pub fn create_system_raw<S>(
        &mut self,
        name: impl Into<String>,
        components: impl IntoIterator<Item = impl Into<component::RawId>>,
        runner: S,
    ) -> Result<system::Id>
    where
        S: SystemRunner,
    {
        self.systems.create_raw(
            name.into(),
            components.into_iter().map(Into::into),
            runner,
        )
    }

    pub fn create_system<S, A, L>(
        &mut self,
        name: impl Into<String>,
        components: L,
        runner: S,
    ) -> Result<system::Id>
    where
        L: IntoComponents + TypedComponentList + TypedEntriesComponents<A>,
        S: for<'c, 'e> FnMut(&'c system::Context, L::Entries<'e>) -> Result<()>,
        S: Clone + Send + Sync + 'static,
    {
        self.systems.create_typed(name.into(), components, runner)
    }

    pub fn get_system(&self, name: &str) -> Result<system::Id> {
        self.systems.id_from_name(name)
    }

    pub fn get_system_name(&self, system: system::Id) -> Result<&str> {
        Ok(self.systems.get(system)?.name())
    }

    pub fn create_value_raw(
        &mut self,
        entity: entity::Id,
        component: component::RawId,
    ) -> Result<()> {
        self.components.create_value(entity, component)?;
        self.entities
            .get_mut(entity)
            .adding_info("component.id", component)?
            .add_component(component)?;
        Ok(())
    }

    pub fn create_value<C>(
        &mut self,
        entity: entity::Id,
        component: component::Id<C>,
        value: C::Value,
    ) -> Result<()>
    where
        C: Component,
    {
        self.create_value_raw(entity, component.raw())?;
        self.set_value(entity, component, value)?;
        Ok(())
    }

    pub fn remove_entity(&mut self, entity: entity::Id) -> Result<()> {
        self.entities.remove(entity)?;
        self.components.remove_values(entity)?;
        Ok(())
    }

    pub fn remove_component(
        &mut self,
        component: component::RawId,
    ) -> Result<()> {
        self.components.remove(component)?;
        for entity in self.entities.iter_mut() {
            entity.remove_component(component)?;
        }
        Ok(())
    }

    pub fn remove_system(&mut self, system: system::Id) -> Result<()> {
        self.systems.remove(system)?;
        Ok(())
    }

    pub fn remove_value(
        &mut self,
        entity: entity::Id,
        component: component::RawId,
    ) -> Result<()> {
        self.components.remove_value(entity, component)?;
        self.entities
            .get_mut(entity)
            .adding_info("component.id", component)?
            .remove_component(component)?;
        Ok(())
    }

    pub fn get_value_raw(
        &self,
        entity: entity::Id,
        component: component::RawId,
    ) -> Result<AnyValue> {
        self.components
            .get(component)
            .adding_info("entity.id", entity)?
            .get(entity)
            .adding_info("component.id", component)
    }

    pub fn set_value_raw(
        &mut self,
        entity: entity::Id,
        component: component::RawId,
        primitive: AnyValue,
    ) -> Result<()> {
        self.components
            .get_mut(component)
            .adding_info("entity.id", entity)?
            .set(entity, primitive)
            .adding_info("component.id", component)
    }

    pub fn get_value<C>(
        &self,
        entity: entity::Id,
        component: component::Id<C>,
    ) -> Result<C::Value>
    where
        C: Component,
    {
        let primitive = self.get_value_raw(entity, component.raw())?;
        <C as Component>::Value::try_from_primitive(primitive)
    }

    pub fn set_value<C>(
        &mut self,
        entity: entity::Id,
        component: component::Id<C>,
        value: C::Value,
    ) -> Result<()>
    where
        C: Component,
    {
        let primitive = value.try_to_primitive()?;
        self.set_value_raw(entity, component.raw(), primitive)
    }

    pub fn tick(&mut self) -> Result<()> {
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
                            .adding_info("system.id", system.id())
                            .adding_info("entity.id", entity.id())?
                            .get(entity.id())
                            .adding_info("system.id", system.id())
                            .adding_info("component.id", component)?;
                        let entry = RawEntry::new(value);
                        entries.push(entry);
                    }
                    let context = system::Context::new(entity.id());
                    system.runner().run(&context, &mut entries)?;
                    for (&component, entry) in
                        system.components().iter().zip(&entries)
                    {
                        self.components
                            .get_mut(component)
                            .adding_info("system.id", system.id())
                            .adding_info("entity.id", entity.id())?
                            .set(entity.id(), entry.get_primitive())
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
