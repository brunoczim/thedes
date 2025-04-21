use serde::{Deserialize, Serialize};
use thedes_geometry::rect;
use thiserror::Error;

use crate::{
    block::{Block, PlaceableBlock},
    geometry::{CoordPair, Rect},
    matter::{Biome, Ground},
};

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Map size {given_size} is below the minimum of {}", Map::MIN_SIZE)]
    TooSmall { given_size: CoordPair },
    #[error("Map rectangle {given_rect} has overflowing bottom right point")]
    BottomRightOverflow { given_rect: Rect },
}

#[derive(Debug, Error)]
#[error("Point is outside of map")]
pub struct InvalidPoint {
    #[from]
    source: rect::HorzAreaError<usize>,
}

#[derive(Debug, Error)]
pub enum AccessError {
    #[error(transparent)]
    InvalidPoint(#[from] InvalidPoint),
    #[error("Bits {1} in point {0} are not valid to decode biome value")]
    GetBiome(CoordPair, u8),
    #[error("Bits {1} in point {0} are not valid to decode ground value")]
    GetGround(CoordPair, u8),
    #[error("Bits {1} in point {0} are not valid to decode block value")]
    GetBlock(CoordPair, u8),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Map {
    rect: Rect,
    biome_layer: Vec<Biome>,
    ground_layer: Vec<Ground>,
    block_layer: Vec<Block>,
}

impl Map {
    pub const MIN_SIZE: CoordPair = CoordPair { x: 100, y: 100 };

    pub fn new(rect: Rect) -> Result<Self, InitError> {
        if rect
            .size
            .zip2(Self::MIN_SIZE)
            .any(|(given, required)| given < required)
        {
            Err(InitError::TooSmall { given_size: rect.size })?
        }
        if rect.checked_bottom_right().is_none() {
            Err(InitError::BottomRightOverflow { given_rect: rect })?
        }

        let total_area = usize::from(rect.map(usize::from).total_area());

        Ok(Self {
            rect,
            biome_layer: vec![Biome::default(); total_area],
            ground_layer: vec![Ground::default(); total_area],
            block_layer: vec![Block::default(); total_area],
        })
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn get_biome(&self, point: CoordPair) -> Result<Biome, AccessError> {
        let index = self.to_flat_index(point)?;
        Ok(self.biome_layer[index])
    }

    pub fn set_biome(
        &mut self,
        point: CoordPair,
        biome: Biome,
    ) -> Result<(), AccessError> {
        let index = self.to_flat_index(point)?;
        self.biome_layer[index] = biome;
        Ok(())
    }

    pub fn get_ground(&self, point: CoordPair) -> Result<Ground, AccessError> {
        let index = self.to_flat_index(point)?;
        Ok(self.ground_layer[index])
    }

    pub fn set_ground(
        &mut self,
        point: CoordPair,
        ground: Ground,
    ) -> Result<(), AccessError> {
        let index = self.to_flat_index(point)?;
        self.ground_layer[index] = ground;
        Ok(())
    }

    pub fn get_block(&self, point: CoordPair) -> Result<Block, AccessError> {
        let index = self.to_flat_index(point)?;
        Ok(self.block_layer[index])
    }

    pub(crate) fn set_block<T>(
        &mut self,
        point: CoordPair,
        block: T,
    ) -> Result<(), AccessError>
    where
        T: Into<Block>,
    {
        let index = self.to_flat_index(point)?;
        self.block_layer[index] = block.into();
        Ok(())
    }

    pub fn set_placeable_block(
        &mut self,
        point: CoordPair,
        block: PlaceableBlock,
    ) -> Result<(), AccessError> {
        self.set_block(point, block)
    }

    fn to_flat_index(&self, point: CoordPair) -> Result<usize, InvalidPoint> {
        let index = self
            .rect
            .map(usize::from)
            .checked_horz_area_down_to(point.map(usize::from))?;
        Ok(index)
    }
}
