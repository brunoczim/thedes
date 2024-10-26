use thedes_domain::{
    geometry::CoordPair,
    map::{AccessError, Map},
    thede,
};

use super::Layer;

pub type ThedeLayerError = AccessError;

#[derive(Debug, Clone)]
pub struct ThedeLayer;

impl Layer for ThedeLayer {
    type Data = Option<thede::Id>;
    type Error = ThedeLayerError;

    fn get(
        &self,
        map: &mut Map,
        point: CoordPair,
    ) -> Result<Self::Data, Self::Error> {
        map.get_thede(point)
    }

    fn set(
        &self,
        map: &mut Map,
        point: CoordPair,
        value: Self::Data,
    ) -> Result<(), Self::Error> {
        map.set_thede(point, value)
    }
}
