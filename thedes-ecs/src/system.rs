use std::{
    collections::{BTreeMap, HashMap, hash_map},
    fmt,
};

use crate::{
    component::{self, Component},
    entity,
    error::{ErrorCause, OptionExt, Result, ResultMapExt, ResultWrapExt},
    value::{Entry, RawEntry},
};

#[derive(Debug)]
pub struct Context {
    entity: entity::Id,
}

impl Context {
    pub(crate) fn new(entity: entity::Id) -> Self {
        Self { entity }
    }

    pub fn entity(&self) -> entity::Id {
        self.entity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl Id {
    pub fn cast_to_index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

pub trait SystemRunner: Send + Sync + 'static {
    fn run<'s, 'c, 'e>(
        &'s mut self,
        ctx: &'c Context,
        values: &'e mut [RawEntry],
    ) -> Result<()>;

    fn dyn_clone(&self) -> Box<dyn SystemRunner>;
}

impl<F> SystemRunner for F
where
    F: for<'c, 'e> FnMut(&'c Context, &'e mut [RawEntry]) -> Result<()>,
    F: Clone + Send + Sync + 'static,
{
    fn run<'s, 'c, 'e>(
        &'s mut self,
        ctx: &'c Context,
        entries: &'e mut [RawEntry],
    ) -> Result<()> {
        self(ctx, entries)
    }

    fn dyn_clone(&self) -> Box<dyn SystemRunner> {
        Box::new(self.clone())
    }
}

impl fmt::Debug for dyn SystemRunner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self as *const dyn SystemRunner)
    }
}

impl Clone for Box<dyn SystemRunner> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

pub trait TypedComponentList {
    type Entries<'b>: TypedEntries<'b>;
}

pub trait TypedSystemRunner<A>: Clone + Send + Sync + 'static
where
    A: TypedComponentList,
{
    fn run<'s, 'c, 'e>(
        &'s mut self,
        ctx: &'c Context,
        entries: <A as TypedComponentList>::Entries<'e>,
    ) -> Result<()>;
}

impl<F, A> TypedSystemRunner<A> for F
where
    A: TypedComponentList,
    F: for<'c, 'e> FnMut(&'c Context, A::Entries<'e>) -> Result<()>,
    F: Clone + Send + Sync + 'static,
{
    fn run<'s, 'c, 'e>(
        &'s mut self,
        ctx: &'c Context,
        entries: <A as TypedComponentList>::Entries<'e>,
    ) -> Result<()> {
        self(ctx, entries)
    }
}

pub trait TypedEntries<'b>: Sized {
    fn try_from_raw(raw: &'b mut [RawEntry]) -> Result<Self>;
}

pub trait TypedEntriesComponents<C>: IntoComponents {}

pub trait IntoComponents {
    type IntoComponents: IntoIterator<Item = component::RawId>;

    fn into_components(self) -> Self::IntoComponents;
}

#[derive(Debug, Clone)]
pub(crate) struct Record {
    id: Id,
    components: Vec<component::RawId>,
    runner: Box<dyn SystemRunner>,
    name: String,
}

impl Record {
    pub fn new<S>(
        id: Id,
        name: String,
        components: impl IntoIterator<Item = component::RawId>,
        runner: S,
    ) -> Self
    where
        S: SystemRunner,
    {
        Self {
            id,
            name,
            components: components.into_iter().collect(),
            runner: Box::new(runner),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn components(&self) -> &[component::RawId] {
        &self.components[..]
    }

    pub fn runner(&mut self) -> &mut dyn SystemRunner {
        &mut *self.runner
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

    pub fn create_raw<S>(
        &mut self,
        name: String,
        components: impl IntoIterator<Item = component::RawId>,
        runner: S,
    ) -> Result<Id>
    where
        S: SystemRunner,
    {
        match self.names.entry(name.clone()) {
            hash_map::Entry::Vacant(entry) => {
                let id = self.next;
                entry.insert(id);
                self.next.0 += 1;
                self.records
                    .insert(id, Record::new(id, name, components, runner));
                Ok(id)
            },

            hash_map::Entry::Occupied(entry) => {
                Err(ErrorCause::DuplicateSystemName)
                    .wrap_ctx()
                    .adding_info("system.name", name)
                    .adding_info("system.conflict.id", entry.get())?
            },
        }
    }

    pub fn create_typed<S, A, C>(
        &mut self,
        name: String,
        components: C,
        mut runner: S,
    ) -> Result<Id>
    where
        S: TypedSystemRunner<C>,
        C: IntoComponents + TypedComponentList + TypedEntriesComponents<A>,
    {
        self.create_raw(
            name,
            components.into_components(),
            move |ctx: &Context, raw_entries: &mut [RawEntry]| {
                let entries = <
                        <C as TypedComponentList>::Entries<'_> as TypedEntries
                    >::try_from_raw(raw_entries)?;
                runner.run(ctx, entries)?;
                Ok(())
            },
        )
    }

    pub fn id_from_name(&self, name: &str) -> Result<Id> {
        self.names
            .get(name)
            .copied()
            .ok_or_ctx(ErrorCause::InvalidSystemName)
            .adding_info("system.name", name)
    }

    pub fn get(&self, system: Id) -> Result<&Record> {
        self.records
            .get(&system)
            .ok_or_ctx(ErrorCause::InvalidSystemId)
            .adding_info("system.id", system)
    }

    #[expect(unused)]
    pub fn get_mut(&mut self, system: Id) -> Result<&mut Record> {
        self.records
            .get_mut(&system)
            .ok_or_ctx(ErrorCause::InvalidSystemId)
            .adding_info("system.id", system)
    }

    pub fn remove(&mut self, system: Id) -> Result<()> {
        self.records
            .remove(&system)
            .ok_or_ctx(ErrorCause::InvalidSystemId)
            .adding_info("system.id", system)?;
        Ok(())
    }

    #[expect(unused)]
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

macro_rules! impl_arity {
    (types = (), arity = $n:expr) => {};

    (types = ($ident:ident $($idents:ident)*), arity = $n:expr) => {
        impl_arity! { @case, types = ($ident $($idents)*), arity = $n }
        impl_arity! { types = ($($idents)*), arity = $n - 1 }
    };

    (@case, types = ($($ident:ident)*), arity = $n:expr) => {
        impl<$($ident,)*> TypedComponentList
            for ($(component::Id<$ident>,)*)
        where
            $($ident: Component,)*
        {
            type Entries<'b> = ($(Entry<'b, $ident>,)*);
        }

        impl<'b, $($ident,)*>
            TypedEntriesComponents<($(Entry<'b, $ident>,)*)>
            for ($(component::Id<$ident>,)*)
        {
        }

        impl<'b, $($ident,)*> IntoComponents
            for ($(component::Id<$ident>,)*)
        {
            type IntoComponents =
                std::array::IntoIter<component::RawId, { $n }>;

            fn into_components(self) -> Self::IntoComponents {
                #[allow(non_snake_case)]
                let ($($ident,)*) = self;
                [$($ident.raw(),)*].into_iter()
            }
        }

        impl<'b, $($ident,)*> TypedEntries<'b> for ($(Entry<'b, $ident>,)*)
        where
            $($ident: Component,)*
        {
            fn try_from_raw(raw: &'b mut [RawEntry])
                -> Result<Self>
            {
                let mut raw_iter = raw.iter_mut();
                Ok(($(
                    Entry::<'_, $ident>::from_raw(
                        raw_iter.next().ok_or_ctx(ErrorCause::MissingEntry)?
                    ),
                )*))
            }
        }
    };
}

impl_arity! {
    types = (
        S0 S1 S2 S3 S4 S5 S6 S7 S8 S9 S10 S11 S12 S13 S14 S15
//        T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15
//        U0 U1 U2 U3 U4 U5 U6 U7 U8 U9 U10 U11 U12 U13 U14 U15
//        V0 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 V13 V14 V15
//        W0 W1 W2 W3 W4 W5 W6 W7 W8 W9 W10 W11 W12 W13 W14 W15
//        X0 X1 X2 X3 X4 X5 X6 X7 X8 X9 X10 X11 X12 X13 X14 X15
//        Y0 Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 Y9 Y10 Y11 Y12 Y13 Y14 Y15
//        Z0 Z1 Z2 Z3 Z4 Z5 Z6 Z7 Z8 Z9 Z10 Z11 Z12 Z13 Z14 Z15
    ),
   // arity = 128
   arity = 16
}
