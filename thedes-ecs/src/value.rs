use std::{error::Error, fmt, marker::PhantomData};

use thiserror::Error;

use crate::error::{CtxResult, ResultMapExt};

pub type AnyValue = u64;

#[derive(Debug, Error)]
pub enum FromPrimitiveError {
    #[error("invalid primitive")]
    Invalid,
    #[error("invalid primitive: {0}")]
    Message(String),
    #[error("invalid primitive")]
    Source(
        #[from]
        #[source]
        Box<dyn Error + Send + Sync>,
    ),
}

#[derive(Debug, Error)]
pub enum ToPrimitiveError {
    #[error("invalid value")]
    Invalid,
    #[error("invalid value: {0}")]
    Message(String),
    #[error("invalid value")]
    Source(
        #[from]
        #[source]
        Box<dyn Error + Send + Sync>,
    ),
}

pub trait TryValue: Sized {
    fn try_from_primitive(
        primitive: AnyValue,
    ) -> CtxResult<Self, FromPrimitiveError>;

    fn try_to_primitive(&self) -> CtxResult<AnyValue, ToPrimitiveError>;
}

pub trait Value: Sized {
    fn from_primitive(primitive: AnyValue) -> Self;

    fn to_primitive(&self) -> AnyValue;
}

impl Value for u64 {
    fn to_primitive(&self) -> AnyValue {
        *self
    }

    fn from_primitive(primitive: AnyValue) -> Self {
        primitive
    }
}

impl Value for f64 {
    fn to_primitive(&self) -> AnyValue {
        self.to_bits()
    }

    fn from_primitive(primitive: AnyValue) -> Self {
        Self::from_bits(primitive)
    }
}

impl<V> TryValue for V
where
    V: Value,
{
    fn try_from_primitive(
        primitive: AnyValue,
    ) -> CtxResult<Self, FromPrimitiveError> {
        Ok(Self::from_primitive(primitive))
    }

    fn try_to_primitive(&self) -> CtxResult<AnyValue, ToPrimitiveError> {
        Ok(self.to_primitive())
    }
}

#[derive(Debug)]
pub struct RawEntry {
    primitive: AnyValue,
}

impl RawEntry {
    pub(crate) fn new(primitive: AnyValue) -> Self {
        Self { primitive }
    }

    pub fn get<V>(&self) -> V
    where
        V: Value,
    {
        V::from_primitive(self.get_primitive())
    }

    pub fn set<V>(&mut self, value: V)
    where
        V: Value,
    {
        self.set_primitive(value.to_primitive());
    }

    pub fn try_get<V>(&self) -> CtxResult<V, FromPrimitiveError>
    where
        V: TryValue,
    {
        V::try_from_primitive(self.get_primitive())
            .adding_info("primitive", self.get_primitive())
    }

    pub fn try_set<V>(&mut self, value: V) -> CtxResult<(), ToPrimitiveError>
    where
        V: TryValue,
    {
        let primitive = value.try_to_primitive()?;
        self.set_primitive(primitive);
        Ok(())
    }

    pub fn get_primitive(&self) -> AnyValue {
        self.primitive
    }

    pub fn set_primitive(&mut self, primitive: AnyValue) {
        self.primitive = primitive;
    }
}

pub struct Entry<'b, V> {
    raw: &'b mut RawEntry,
    _marker: PhantomData<[V; 0]>,
}

impl<'b, V> fmt::Debug for Entry<'b, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry").field("raw", &self.raw).finish()
    }
}

impl<'b, V> Entry<'b, V> {
    pub fn from_raw(raw: &'b mut RawEntry) -> Self {
        Self { raw, _marker: PhantomData }
    }

    pub fn into_raw(self) -> &'b mut RawEntry {
        self.raw
    }

    pub fn get(&self) -> V
    where
        V: Value,
    {
        self.raw.get()
    }

    pub fn set(&mut self, value: V)
    where
        V: Value,
    {
        self.raw.set(value);
    }

    pub fn try_get(&self) -> CtxResult<V, FromPrimitiveError>
    where
        V: TryValue,
    {
        self.raw.try_get()
    }

    pub fn try_set(&mut self, value: V) -> CtxResult<(), ToPrimitiveError>
    where
        V: TryValue,
    {
        self.raw.try_set(value)
    }
}
