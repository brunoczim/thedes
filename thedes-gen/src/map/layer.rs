use rand::Rng;
use thedes_domain::{geometry::CoordPair, map::Map};

pub mod region;
pub mod pointwise;

pub mod matter;
pub mod block;

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

pub trait LayerDistribution {
    type Data;
    type Error;

    fn sample<R>(
        &self,
        map: &mut Map,
        point: CoordPair,
        rng: R,
    ) -> Result<Self::Data, Self::Error>
    where
        R: Rng;
}
