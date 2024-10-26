use thedes_geometry::rect;
use thiserror::Error;

use crate::{
    bitpack::{self, BitPack, BitVector},
    block::{Block, PlaceableBlock},
    geometry::{CoordPair, Rect},
    matter::{Biome, Ground},
    thede,
};

#[derive(Debug, Error)]
pub enum CreationError {
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

#[derive(Debug, Clone)]
pub struct Map {
    rect: Rect,
    biome_layer: Box<[u8]>,
    ground_layer: Box<[u8]>,
    block_layer: Box<[u8]>,
    thede_layer: Box<[u8]>,
}

impl Map {
    pub const MIN_SIZE: CoordPair = CoordPair { x: 100, y: 100 };

    pub fn new(rect: Rect) -> Result<Self, CreationError> {
        if rect
            .size
            .zip2(Self::MIN_SIZE)
            .any(|(given, required)| given < required)
        {
            Err(CreationError::TooSmall { given_size: rect.size })?
        }
        if rect.checked_bottom_right().is_none() {
            Err(CreationError::BottomRightOverflow { given_rect: rect })?
        }

        let total_area = rect.map(usize::from).total_area();

        let biomes_per_byte =
            <<Biome as BitPack>::BitVector as BitVector>::BIT_COUNT
                / Biome::BIT_COUNT;
        let biomes_per_byte = biomes_per_byte as usize;
        let ceiled_area = total_area + biomes_per_byte;
        let biome_buf_size = ceiled_area / biomes_per_byte;

        let grounds_per_byte =
            <<Ground as BitPack>::BitVector as BitVector>::BIT_COUNT
                / Ground::BIT_COUNT;
        let grounds_per_byte = grounds_per_byte as usize;
        let ceiled_area = total_area + grounds_per_byte;
        let ground_buf_size = ceiled_area / grounds_per_byte;

        let blocks_per_byte =
            <<Block as BitPack>::BitVector as BitVector>::BIT_COUNT
                / Block::BIT_COUNT;
        let blocks_per_byte = blocks_per_byte as usize;
        let ceiled_area = total_area + blocks_per_byte;
        let block_buf_size = ceiled_area / blocks_per_byte;

        let thede_buf_size = total_area;

        Ok(Self {
            rect,
            biome_layer: Box::from(vec![0; biome_buf_size]),
            ground_layer: Box::from(vec![0; ground_buf_size]),
            block_layer: Box::from(vec![0; block_buf_size]),
            thede_layer: Box::from(vec![0; thede_buf_size]),
        })
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn get_biome(&self, point: CoordPair) -> Result<Biome, AccessError> {
        let index = self.to_flat_index(point)?;
        bitpack::read_packed(&self.biome_layer, index)
            .map_err(|bits| AccessError::GetBiome(point, bits))
    }

    pub fn set_biome(
        &mut self,
        point: CoordPair,
        biome: Biome,
    ) -> Result<(), AccessError> {
        let index = self.to_flat_index(point)?;
        bitpack::write_packed(&mut self.biome_layer, index, biome);
        Ok(())
    }

    pub fn get_ground(&self, point: CoordPair) -> Result<Ground, AccessError> {
        let index = self.to_flat_index(point)?;
        bitpack::read_packed(&self.ground_layer, index)
            .map_err(|bits| AccessError::GetGround(point, bits))
    }

    pub fn set_ground(
        &mut self,
        point: CoordPair,
        ground: Ground,
    ) -> Result<(), AccessError> {
        let index = self.to_flat_index(point)?;
        bitpack::write_packed(&mut self.ground_layer, index, ground);
        Ok(())
    }

    pub fn get_block(&self, point: CoordPair) -> Result<Block, AccessError> {
        let index = self.to_flat_index(point)?;
        bitpack::read_packed(&self.block_layer, index)
            .map_err(|bits| AccessError::GetBlock(point, bits))
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
        bitpack::write_packed(&mut self.block_layer, index, block.into());
        Ok(())
    }

    pub fn set_placeable_block(
        &mut self,
        point: CoordPair,
        block: PlaceableBlock,
    ) -> Result<(), AccessError> {
        self.set_block(point, block)
    }

    pub fn get_thede(
        &self,
        point: CoordPair,
    ) -> Result<Option<thede::Id>, AccessError> {
        let index = self.to_flat_index(point)?;
        Ok(thede::Id::new(self.thede_layer[index]))
    }

    pub fn set_thede(
        &mut self,
        point: CoordPair,
        thede: Option<thede::Id>,
    ) -> Result<(), AccessError> {
        let index = self.to_flat_index(point)?;
        self.thede_layer[index] = thede::Id::option_bits(thede);
        Ok(())
    }

    pub fn put_thede(
        &mut self,
        point: CoordPair,
        thede: thede::Id,
    ) -> Result<(), AccessError> {
        self.set_thede(point, Some(thede))
    }

    pub fn clear_thede(&mut self, point: CoordPair) -> Result<(), AccessError> {
        self.set_thede(point, None)
    }

    fn to_flat_index(&self, point: CoordPair) -> Result<usize, InvalidPoint> {
        let index = self
            .rect
            .map(usize::from)
            .checked_horz_area_down_to(point.map(usize::from))?;
        Ok(index)
    }
}
