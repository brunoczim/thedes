use thedes_domain::{
    block::Block,
    geometry::CoordPair,
    map::{AccessError, Map},
};
use thiserror::Error;

use super::{Layer, LayerDistribution};

#[derive(Debug, Error)]
pub enum BlockLayerError {
    #[error("Failed to access map position")]
    Access(
        #[from]
        #[source]
        AccessError,
    ),
    #[error("Attempted to set forbidden block type at {0}")]
    Forbidden(CoordPair),
}

#[derive(Debug, Clone)]
pub struct BlockLayer;

impl Layer for BlockLayer {
    type Data = Block;
    type Error = BlockLayerError;

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
        let block = value
            .placeable()
            .ok_or_else(|| BlockLayerError::Forbidden(point))?;
        map.set_placeable_block(point, block)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BlockLayerDistribution {}

impl LayerDistribution for BlockLayerDistribution {
    type Data = Block;
    type Error = ();

    fn sample<R>(
        &self,
        map: &mut Map,
        point: CoordPair,
        rng: R,
    ) -> Result<Self::Data, Self::Error>
    where
        R: rand::Rng,
    {
        todo!()
    }
}
