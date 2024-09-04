use std::array;

use thedes_domain::{
    bitpack::BitPack,
    block::{Block, PlaceableBlock},
    geometry::CoordPair,
    map::{AccessError, Map},
    matter::Ground,
};
use thiserror::Error;

use crate::{map::layer::matter::GroundLayer, random::ProabilityWeight};

use super::{Layer, LayerDistribution};

#[derive(Debug, Error)]
pub enum LayerError {
    #[error("Failed to access map position")]
    Access(
        #[from]
        #[source]
        AccessError,
    ),
    #[error("Attempted to set forbidden block type at {0}")]
    Forbidden(CoordPair),
}

#[derive(Debug, Error)]
pub enum DistError {
    #[error("Failed to access map position")]
    Access(
        #[from]
        #[source]
        AccessError,
    ),
    #[error("Internal error")]
    BadDist,
}

#[derive(Debug, Clone)]
pub struct BlockLayer;

impl Layer for BlockLayer {
    type Data = Block;
    type Error = LayerError;

    fn get(
        &self,
        map: &mut Map,
        point: CoordPair,
    ) -> Result<Self::Data, Self::Error> {
        Ok(map.get_block(point)?)
    }

    fn set(
        &self,
        map: &mut Map,
        point: CoordPair,
        value: Self::Data,
    ) -> Result<(), Self::Error> {
        let block =
            value.placeable().ok_or_else(|| LayerError::Forbidden(point))?;
        map.set_placeable_block(point, block)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BlockLayerDist {
    cumulative_weights:
        [[ProabilityWeight; PlaceableBlock::ELEM_COUNT]; Ground::ELEM_COUNT],
}

impl Default for BlockLayerDist {
    fn default() -> Self {
        Self::new(|ground, block| match (ground, block) {
            (Ground::Grass, PlaceableBlock::Air) => 2094,
            (Ground::Grass, PlaceableBlock::Stick) => 1,
            (Ground::Sand, PlaceableBlock::Air) => 10139,
            (Ground::Sand, PlaceableBlock::Stick) => 1,
            (Ground::Stone, PlaceableBlock::Air) => 1,
            (Ground::Stone, PlaceableBlock::Stick) => 0,
        })
    }
}

impl BlockLayerDist {
    pub fn new<F>(mut density_function: F) -> Self
    where
        F: FnMut(Ground, PlaceableBlock) -> ProabilityWeight,
    {
        let cumulative_weights = array::from_fn(|i| {
            let mut accumuled_weight = 0;
            array::from_fn(|j| {
                accumuled_weight +=
                    density_function(Ground::ALL[i], PlaceableBlock::ALL[j]);
                accumuled_weight
            })
        });
        Self { cumulative_weights }
    }
}

impl LayerDistribution for BlockLayerDist {
    type Data = Block;
    type Error = DistError;

    fn sample<R>(
        &self,
        map: &mut Map,
        point: CoordPair,
        mut rng: R,
    ) -> Result<Self::Data, Self::Error>
    where
        R: rand::Rng,
    {
        let ground = GroundLayer.get(map, point)?;
        let cumulative_weights =
            self.cumulative_weights[usize::from(ground as u8)];
        let last_cumulative_weight =
            cumulative_weights[cumulative_weights.len() - 1];
        let sampled_weight = rng.gen_range(0 .. last_cumulative_weight);
        for (i, cumulative_weight) in cumulative_weights.into_iter().enumerate()
        {
            if sampled_weight < cumulative_weight {
                return Ok(PlaceableBlock::ALL[i].into());
            }
        }
        Err(DistError::BadDist)
    }
}
