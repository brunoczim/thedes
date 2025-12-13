use std::{collections::HashMap, fmt};

use thiserror::Error;

use crate::{
    component,
    error::{CtxResult, OptionExt, ResultMapExt},
    value::{Entry, RawEntry, TryValue},
    world::Error,
};

#[derive(Debug, Error)]
pub enum GetError {
    #[error("system identifier is invalid")]
    Invalid,
}

#[derive(Debug, Error)]
pub enum RemoveError {
    #[error("system identifier is invalid")]
    Invalid,
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
    fn run<'a, 'b>(
        &'a mut self,
        values: &'b mut [RawEntry],
    ) -> CtxResult<(), Error>;

    fn dyn_clone(&self) -> Box<dyn SystemRunner>;
}

impl<F> SystemRunner for F
where
    F: for<'b> FnMut(&'b mut [RawEntry]) -> CtxResult<(), Error>,
    F: Clone + Send + Sync + 'static,
{
    fn run<'a, 'b>(
        &'a mut self,
        entries: &'b mut [RawEntry],
    ) -> CtxResult<(), Error> {
        self(entries)
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
    fn run<'a, 'b>(
        &'a mut self,
        entries: <A as TypedComponentList>::Entries<'b>,
    ) -> CtxResult<(), Error>;
}

pub trait TypedEntries<'b>: Sized {
    fn try_from_raw(raw: &'b mut [RawEntry]) -> CtxResult<Self, Error>;
}

pub trait TypedEntriesArity<C>
where
    C: IntoComponents,
{
}

pub trait IntoComponents: IntoIterator<Item = component::Id> {}

impl<C> IntoComponents for C where C: IntoIterator<Item = component::Id> {}

#[derive(Debug, Clone)]
pub(crate) struct Record {
    id: Id,
    components: Vec<component::Id>,
    runner: Box<dyn SystemRunner>,
}

impl Record {
    pub fn new<S>(
        id: Id,
        components: impl IntoIterator<Item = component::Id>,
        runner: S,
    ) -> Self
    where
        S: SystemRunner,
    {
        Self {
            id,
            components: components.into_iter().collect(),
            runner: Box::new(runner),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn components(&self) -> &[component::Id] {
        &self.components[..]
    }

    pub fn runner(&mut self) -> &mut dyn SystemRunner {
        &mut *self.runner
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

    pub fn create_raw<S>(
        &mut self,
        components: impl IntoIterator<Item = component::Id>,
        runner: S,
    ) -> Id
    where
        S: SystemRunner,
    {
        let id = self.next;
        self.next.0 += 1;
        self.records.insert(id, Record::new(id, components, runner));
        id
    }

    pub fn create_typed<S, A, C>(&mut self, components: C, mut runner: S) -> Id
    where
        S: TypedSystemRunner<A>,
        A: TypedComponentList + TypedEntriesArity<C>,
        C: IntoComponents,
    {
        let id = self.next;
        self.next.0 += 1;
        self.records.insert(
            id,
            Record::new(
                id,
                components,
                move |raw_entries: &mut [RawEntry]| {
                    let entries = A::try_from_raw(raw_entries)?;
                    runner.run(entries)?;
                    Ok(())
                },
            ),
        );
        id
    }

    #[expect(unused)]
    pub fn get(&self, system: Id) -> CtxResult<&Record, GetError> {
        self.records
            .get(&system)
            .ok_or_ctx(GetError::Invalid)
            .adding_info("system.id", system)
    }

    #[expect(unused)]
    pub fn get_mut(&mut self, system: Id) -> CtxResult<&mut Record, GetError> {
        self.records
            .get_mut(&system)
            .ok_or_ctx(GetError::Invalid)
            .adding_info("system.id", system)
    }

    pub fn remove(&mut self, system: Id) -> CtxResult<(), RemoveError> {
        self.records
            .remove(&system)
            .ok_or_ctx(RemoveError::Invalid)
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
        impl<'b, $($ident,)*> TypedEntriesArity<[component::Id; $n]>
            for ($(Entry<'b, $ident>,)*)
        {
        }

        impl<'b, $($ident,)*> TypedEntries<'b> for ($(Entry<'b, $ident>,)*)
        where
            $($ident: TryValue,)*
        {
            fn try_from_raw(raw: &'b mut [RawEntry])
                -> CtxResult<Self, Error>
            {
                let mut raw_iter = raw.iter_mut();
                Ok(($(
                    Entry::<'_, $ident>::from_raw(
                        raw_iter.next().ok_or_ctx(Error::MissingEntry)?
                    ),
                )*))
            }
        }
    };
}

impl_arity! {
    types = (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15),
    arity = 16
}
