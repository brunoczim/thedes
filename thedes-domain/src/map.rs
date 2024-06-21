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

#[derive(Debug, Error)]
pub enum InvalidRect {
    #[error("Map size {given_size} is below the minimum of {}", Map::MIN_SIZE)]
    TooSmall { given_size: CoordPair },
    #[error("Map rectangle {given_rect} has overflowing bottom right point")]
    BottomRightOverflow { given_rect: Rect },
    #[error("Map rectangle size {given_size} has overflowing area")]
    AreaOverflow { given_size: CoordPair },
}

#[derive(Debug, Clone)]
pub struct MapConfig {
    rect: Rect,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl MapConfig {
    pub fn new() -> Self {
        Self {
            rect: Rect {
                top_left: thedes_geometry::CoordPair { y: 0, x: 0 },
                size: CoordPair { y: 1000, x: 1000 },
            },
        }
    }

    pub fn with_rect(self, rect: Rect) -> Result<Self, InvalidRect> {
        if rect.size.zip2(Map::MIN_SIZE).any(|(given, min)| given < min) {
            Err(InvalidRect::TooSmall { given_size: rect.size })?
        }
        if rect.checked_bottom_right().is_none() {
            Err(InvalidRect::BottomRightOverflow { given_rect: rect })?
        }
        Ok(Self { rect, ..self })
    }

    pub(crate) fn finish(self) -> Map {
        Map::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    config: MapConfig,
    ground_layer: Box<[Ground]>,
}

impl Map {
    pub const MIN_SIZE: CoordPair = CoordPair { x: 64, y: 64 };

    fn new(config: MapConfig) -> Self {
        let buf_size = config.rect.map(usize::from).total_area();

        Self {
            config,
            ground_layer: Box::from(vec![Ground::default(); buf_size]),
        }
    }

    pub fn rect(&self) -> Rect {
        self.config.rect
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
            .config
            .rect
            .map(usize::from)
            .checked_horz_area_down_to(point.map(usize::from))?;
        Ok(index)
    }
}
