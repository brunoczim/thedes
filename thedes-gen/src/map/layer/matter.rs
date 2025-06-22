use thedes_domain::{
    geometry::CoordPair,
    map::{AccessError, Map},
    matter::{Biome, Ground},
};

use super::{Layer, LayerDistribution};

pub type GroundLayerError = AccessError;
pub type BiomeLayerError = AccessError;
pub type GroundDistrError = AccessError;

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

#[derive(Debug, Clone)]
pub struct BiomeLayer;

impl Layer for BiomeLayer {
    type Data = Biome;
    type Error = BiomeLayerError;

    fn get(
        &self,
        map: &mut Map,
        point: CoordPair,
    ) -> Result<Self::Data, Self::Error> {
        map.get_biome(point)
    }

    fn set(
        &self,
        map: &mut Map,
        point: CoordPair,
        value: Self::Data,
    ) -> Result<(), Self::Error> {
        map.set_biome(point, value)
    }
}

#[derive(Debug, Clone)]
pub struct GroundLayerDistr {
    _private: (),
}

impl Default for GroundLayerDistr {
    fn default() -> Self {
        Self { _private: () }
    }
}

impl LayerDistribution for GroundLayerDistr {
    type Data = Ground;
    type Error = GroundDistrError;

    fn sample<R>(
        &self,
        map: &mut Map,
        point: CoordPair,
        _rng: R,
    ) -> Result<Self::Data, Self::Error>
    where
        R: rand::Rng,
    {
        let ground = match map.get_biome(point)? {
            Biome::Plains => Ground::Grass,
            Biome::Desert => Ground::Sand,
            Biome::Wasteland => Ground::Stone,
        };
        Ok(ground)
    }
}
