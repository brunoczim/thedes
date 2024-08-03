use thedes_domain::{
    geometry::CoordPair,
    map::{AccessError, Map},
    matter::Ground,
};

use super::Layer;

pub type GroundLayerError = AccessError;

#[derive(Debug, Clone)]
pub struct GroundLayer;

impl Layer for GroundLayer {
    type Data = Ground;
    type Error = GroundLayerError;

    fn get(
        &self,
        map: &mut Map,
        point: CoordPair,
    ) -> Result<Self::Data, Self::Error> {
        map.get_ground(point)
    }

    fn set(
        &self,
        map: &mut Map,
        point: CoordPair,
        value: Self::Data,
    ) -> Result<(), Self::Error> {
        map.set_ground(point, value)
    }
}
