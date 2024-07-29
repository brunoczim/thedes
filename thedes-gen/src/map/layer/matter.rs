use thedes_domain::{
    geometry::{CoordPair, Rect},
    map::{AccessError, Map},
    matter::Ground,
};

use super::Layer;

pub type GroundLayerError = AccessError;

#[derive(Debug)]
pub struct GroundLayer<'a> {
    map: &'a mut Map,
}

impl<'a> GroundLayer<'a> {
    pub fn new(map: &'a mut Map) -> Self {
        Self { map }
    }
}

impl<'a> Layer for GroundLayer<'a> {
    type Data = Ground;
    type Error = GroundLayerError;

    fn rect(&self) -> Rect {
        self.map.rect()
    }

    fn set(
        &mut self,
        position: CoordPair,
        value: Self::Data,
    ) -> Result<(), Self::Error> {
        self.map.set_ground(position, value)
    }

    fn get(&mut self, position: CoordPair) -> Result<Self::Data, Self::Error> {
        self.map.get_ground(position)
    }
}
