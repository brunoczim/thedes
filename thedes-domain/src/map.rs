use thedes_geometry::rect;
use thiserror::Error;

use crate::{
    geometry::{CoordPair, Rect},
    matter::Ground,
};

#[derive(Debug, Error)]
#[error("Point is outside of map")]
pub struct InvalidPoint {
    #[from]
    source: rect::HorzAreaError<usize>,
}

#[derive(Debug, Clone)]
pub struct Map {
    rect: Rect,
    ground_layer: Box<[Ground]>,
}

impl Map {
    pub const MIN_SIZE: CoordPair = CoordPair { x: 100, y: 100 };

    pub(crate) fn new(rect: Rect) -> Self {
        let buf_size = rect.map(usize::from).total_area();

        Self {
            rect,
            ground_layer: Box::from(vec![Ground::default(); buf_size]),
        }
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn get_ground(&self, point: CoordPair) -> Result<Ground, InvalidPoint> {
        let index = self.to_flat_index(point)?;
        Ok(self.ground_layer[index])
    }

    pub fn set_ground(
        &mut self,
        point: CoordPair,
        value: Ground,
    ) -> Result<(), InvalidPoint> {
        let index = self.to_flat_index(point)?;
        self.ground_layer[index] = value;
        Ok(())
    }

    fn to_flat_index(&self, point: CoordPair) -> Result<usize, InvalidPoint> {
        let index = self
            .rect
            .map(usize::from)
            .checked_horz_area_down_to(point.map(usize::from))?;
        Ok(index)
    }
}
