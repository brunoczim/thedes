use thedes_domain::{geometry::CoordPair, map::Map};

pub mod region;
pub mod matter;

pub trait Layer {
    type Data;
    type Error;

    fn get(
        &self,
        map: &mut Map,
        point: CoordPair,
    ) -> Result<Self::Data, Self::Error>;

    fn set(
        &self,
        map: &mut Map,
        point: CoordPair,
        value: Self::Data,
    ) -> Result<(), Self::Error>;
}
