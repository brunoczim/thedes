use num::FromPrimitive;
use thedes_geometry::rect;
use thiserror::Error;

use crate::{
    geometry::{CoordPair, Rect},
    matter::Ground,
};

const GROUND_BIT_COUNT: usize = 4;
const GROUNDS_PER_BYTE: usize = 8 / GROUND_BIT_COUNT;

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
    #[error("Bits {1} in point {0} are not valid to decode ground value")]
    GetGround(CoordPair, u8),
}

#[derive(Debug, Clone)]
pub struct Map {
    rect: Rect,
    ground_layer: Box<[u8]>,
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
        let ceiled_area = total_area + (GROUNDS_PER_BYTE - 1);
        let buf_size = ceiled_area / GROUNDS_PER_BYTE;

        Ok(Self { rect, ground_layer: Box::from(vec![0; buf_size]) })
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn get_ground(&self, point: CoordPair) -> Result<Ground, AccessError> {
        let index = self.to_flat_index(point)?;
        let (byte_index, ground_index) = Self::split_ground_index(index);
        let shift = ground_index * GROUND_BIT_COUNT;
        let mask = ((1 << GROUND_BIT_COUNT) - 1) as u8;
        let bits = (self.ground_layer[byte_index] >> shift) & mask;
        Ground::from_u8(bits).ok_or(AccessError::GetGround(point, bits))
    }

    pub fn set_ground(
        &mut self,
        point: CoordPair,
        value: Ground,
    ) -> Result<(), AccessError> {
        let index = self.to_flat_index(point)?;
        let (byte_index, ground_index) = Self::split_ground_index(index);
        let shift = ground_index * GROUND_BIT_COUNT;
        let mask = (((1 << GROUND_BIT_COUNT) - 1) as u8) << shift;
        let bits = (value as u8) << shift;
        let previous = self.ground_layer[byte_index];
        self.ground_layer[byte_index] = (previous & !mask) | bits;
        Ok(())
    }

    fn split_ground_index(index: usize) -> (usize, usize) {
        (index / GROUNDS_PER_BYTE, index % GROUNDS_PER_BYTE)
    }

    fn to_flat_index(&self, point: CoordPair) -> Result<usize, InvalidPoint> {
        let index = self
            .rect
            .map(usize::from)
            .checked_horz_area_down_to(point.map(usize::from))?;
        Ok(index)
    }
}
