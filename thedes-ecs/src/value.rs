use std::{fmt, marker::PhantomData};

use crate::{
    component::Component,
    error::{ErrorCause, Result, ResultMapExt, ResultWrapExt},
};

pub type AnyValue = u64;

macro_rules! impl_value_cast {
    () => {};
    ($ty:ty $(, $tys:ty)* $(,)?) => {
        impl Value for $ty {
            fn from_primitive(primitive: AnyValue) -> Self {
                primitive as Self
            }

            fn to_primitive(&self) -> AnyValue {
                *self as AnyValue
            }
        }

        impl_value_cast! { $($tys,)* }
    };
}

macro_rules! impl_try_value_enum {
    ($ty:ty { $($var:ident),* $(,)? }) => {
        impl TryValue for $ty {
            fn try_from_primitive(
                primitive: AnyValue,
            ) -> Result<Self> {
                #[allow(non_upper_case_globals)]
                mod as_const {
                    use super::AnyValue;

                    $(pub const $var: AnyValue = <$ty>::$var as AnyValue;)*
                }

                Ok(match primitive {
                    $(as_const::$var => Self::$var,)*
                    _ => Err(ErrorCause::InvalidPrimitive).wrap_ctx()?,
                })
            }

            fn try_to_primitive(&self) -> Result<AnyValue> {
                Ok(*self as AnyValue)
            }
        }
    };
}

macro_rules! impl_value_coord_pair {
    () => {};

    ($param:ident $(, $params:ident)* $(,)?) => {
        impl Value for thedes_geometry::CoordPair<$param> {
            fn from_primitive(primitive: AnyValue) -> Self {
                let mask = (1 << <$param>::BITS) - 1;
                let y = <$param>::from_primitive(primitive & mask);
                let x = <$param>::from_primitive(
                    (primitive >> <$param>::BITS) & mask,
                );
                Self { y, x }
            }

            fn to_primitive(&self) -> AnyValue {
                let y = self.y.to_primitive();
                let x = self.x.to_primitive();
                y | (x << <$param>::BITS)
            }
        }

        impl_value_coord_pair! { $($params,)* }
    };
}

macro_rules! impl_value_rect {
    () => {};

    ($param:ident $(, $params:ident)* $(,)?) => {
        impl Value for thedes_geometry::rect::Rect<$param> {
            fn from_primitive(primitive: AnyValue) -> Self {
                let mask = (1 << (<$param>::BITS * 2)) - 1;
                let top_left = thedes_geometry::CoordPair::<$param>
                    ::from_primitive(primitive & mask);
                let size = thedes_geometry::CoordPair::<$param>
                    ::from_primitive(
                        (primitive >> (<$param>::BITS * 2)) & mask,
                    );
                Self { top_left, size }
            }

            fn to_primitive(&self) -> AnyValue {
                let top_left = self.top_left.to_primitive();
                let size = self.size.to_primitive();
                top_left | (size << (<$param>::BITS * 2))
            }
        }

        impl_value_rect! { $($params,)* }
    };
}

impl_value_cast! { u8, i8, u16, i16, u32, i32, u64, i64 }

impl_try_value_enum! {
    thedes_geometry::orientation::Direction {
        Up,
        Down,
        Left,
        Right,
    }
}

impl_try_value_enum! {
    thedes_geometry::orientation::Axis {
        X,
        Y,
    }
}

impl_try_value_enum! {
    thedes_geometry::orientation::Order {
        Backwards,
        Forwards,
    }
}

impl_value_coord_pair! { u8, i8, u16, i16, u32, i32 }

impl_value_rect! { u8, i8, u16, i16 }

pub trait TryValue: Sized {
    fn try_from_primitive(primitive: AnyValue) -> Result<Self>;

    fn try_to_primitive(&self) -> Result<AnyValue>;
}

pub trait Value: Sized {
    fn from_primitive(primitive: AnyValue) -> Self;

    fn to_primitive(&self) -> AnyValue;
}

impl<V> TryValue for V
where
    V: Value,
{
    fn try_from_primitive(primitive: AnyValue) -> Result<Self> {
        Ok(Self::from_primitive(primitive))
    }

    fn try_to_primitive(&self) -> Result<AnyValue> {
        Ok(self.to_primitive())
    }
}

impl Value for bool {
    fn to_primitive(&self) -> AnyValue {
        if *self { 1 } else { 0 }
    }

    fn from_primitive(primitive: AnyValue) -> Self {
        primitive != 0
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

    pub fn try_get<V>(&self) -> Result<V>
    where
        V: TryValue,
    {
        V::try_from_primitive(self.get_primitive())
            .adding_info("primitive", self.get_primitive())
    }

    pub fn try_set<V>(&mut self, value: V) -> Result<()>
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

pub struct Entry<'b, C> {
    raw: &'b mut RawEntry,
    _marker: PhantomData<[C; 0]>,
}

impl<'b, C> fmt::Debug for Entry<'b, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry").field("raw", &self.raw).finish()
    }
}

impl<'b, C> Entry<'b, C> {
    pub fn from_raw(raw: &'b mut RawEntry) -> Self {
        Self { raw, _marker: PhantomData }
    }

    pub fn into_raw(self) -> &'b mut RawEntry {
        self.raw
    }

    pub fn get(&self) -> C::Value
    where
        C: Component,
        C::Value: Value,
    {
        self.raw.get()
    }

    pub fn set(&mut self, value: C::Value)
    where
        C: Component,
        C::Value: Value,
    {
        self.raw.set(value);
    }

    pub fn try_get(&self) -> Result<C::Value>
    where
        C: Component,
    {
        self.raw.try_get()
    }

    pub fn try_set(&mut self, value: C::Value) -> Result<()>
    where
        C: Component,
    {
        self.raw.try_set(value)
    }
}
