use crate::geometry::{CoordPair, Rect};

pub mod region;
pub mod matter;

pub trait Layer {
    type Data;
    type Error;

    fn rect(&self) -> Rect;

    fn get(&mut self, position: CoordPair) -> Result<Self::Data, Self::Error>;

    fn set(
        &mut self,
        position: CoordPair,
        value: Self::Data,
    ) -> Result<(), Self::Error>;
}
