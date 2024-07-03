use rand::Rng;
use rand_distr::{Triangular, TriangularError};
use thedes_geometry::axis::{Axis, Direction};
use thiserror::Error;

use crate::{
    game::Game,
    geometry::{Coord, CoordPair, Rect},
    map::Map,
    player::Player,
};

pub type PickedReproducibleRng = rand_chacha::ChaCha8Rng;

#[derive(Debug, Error)]
pub enum InvalidMapConfig {
    #[error("Map size {given_size} is below the minimum of {}", Map::MIN_SIZE)]
    TooSmall { given_size: CoordPair },
    #[error("Map rectangle {given_rect} has overflowing bottom right point")]
    BottomRightOverflow { given_rect: Rect },
    #[error("Map rectangle size {given_size} has overflowing area")]
    AreaOverflow { given_size: CoordPair },
    #[error("Minimum map top left {min} cannot be greater than maximum {max}")]
    TopLeftBoundOrder { min: CoordPair, max: CoordPair },
    #[error("Minimum map size {min} cannot be greater than maximum {max}")]
    SizeBoundOrder { min: CoordPair, max: CoordPair },
}

#[derive(Debug, Error)]
pub enum MapGenError {
    #[error("Error creating random distribution for map top left's axis {1}")]
    TopLeftDist(#[source] TriangularError, Axis),
    #[error("Error creating random distribution for map size's axis {1}")]
    SizeDist(#[source] TriangularError, Axis),
}

#[derive(Debug, Error)]
pub enum GameGenError {
    #[error("Error generating map")]
    Map(
        #[from]
        #[source]
        MapGenError,
    ),
    #[error(
        "Error creating random distribution for player's head in axis {1}"
    )]
    PlayerHeadDist(#[source] TriangularError, Axis),
}

#[derive(Debug, Clone)]
pub struct MapConfig {
    min_top_left: CoordPair,
    max_top_left: CoordPair,
    min_size: CoordPair,
    max_size: CoordPair,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl MapConfig {
    pub fn new() -> Self {
        Self {
            min_top_left: CoordPair { y: 0, x: 0 },
            max_top_left: CoordPair { y: 10_000, x: 10_000 },
            min_size: CoordPair { y: 950, x: 950 },
            max_size: CoordPair { y: 1050, x: 1050 },
        }
    }

    pub fn with_min_top_left(
        self,
        min_top_left: CoordPair,
    ) -> Result<Self, InvalidMapConfig> {
        if min_top_left.zip2(self.max_top_left).any(|(min, max)| min > max) {
            Err(InvalidMapConfig::TopLeftBoundOrder {
                min: min_top_left,
                max: self.max_top_left,
            })?;
        }
        let rect = Rect { top_left: min_top_left, size: self.max_size };
        if rect.checked_bottom_right().is_none() {
            Err(InvalidMapConfig::BottomRightOverflow { given_rect: rect })?
        }
        Ok(Self { min_top_left, ..self })
    }

    pub fn with_max_top_left(
        self,
        max_top_left: CoordPair,
    ) -> Result<Self, InvalidMapConfig> {
        if self.min_top_left.zip2(max_top_left).any(|(min, max)| min > max) {
            Err(InvalidMapConfig::TopLeftBoundOrder {
                min: self.min_top_left,
                max: max_top_left,
            })?;
        }
        let rect = Rect { top_left: max_top_left, size: self.max_size };
        if rect.checked_bottom_right().is_none() {
            Err(InvalidMapConfig::BottomRightOverflow { given_rect: rect })?
        }
        Ok(Self { max_top_left, ..self })
    }

    pub fn with_min_size(
        self,
        min_size: CoordPair,
    ) -> Result<Self, InvalidMapConfig> {
        if min_size.zip2(self.max_size).any(|(min, max)| min > max) {
            Err(InvalidMapConfig::SizeBoundOrder {
                min: min_size,
                max: self.max_size,
            })?;
        }
        let rect = Rect { top_left: self.max_top_left, size: min_size };
        if rect.checked_bottom_right().is_none() {
            Err(InvalidMapConfig::BottomRightOverflow { given_rect: rect })?
        }
        Ok(Self { min_size, ..self })
    }

    pub fn with_max_size(
        self,
        max_size: CoordPair,
    ) -> Result<Self, InvalidMapConfig> {
        if self.min_size.zip2(max_size).any(|(min, max)| min > max) {
            Err(InvalidMapConfig::SizeBoundOrder {
                min: self.min_size,
                max: max_size,
            })?;
        }
        let rect = Rect { top_left: self.max_top_left, size: max_size };
        if rect.checked_bottom_right().is_none() {
            Err(InvalidMapConfig::BottomRightOverflow { given_rect: rect })?
        }
        Ok(Self { max_size, ..self })
    }

    fn generate(
        &self,
        rng: &mut PickedReproducibleRng,
    ) -> Result<Map, MapGenError> {
        let top_left_dist = self
            .min_top_left
            .zip2_with_axes(self.max_top_left, |min, max, axis| {
                let min = f64::from(min);
                let max = f64::from(max) + 1.0 - f64::EPSILON;
                let mode = min + (max - min) / 2.0;
                Triangular::new(min, max, mode)
                    .map_err(|error| MapGenError::TopLeftDist(error, axis))
            })
            .transpose()?;
        let size_dist = self
            .min_size
            .zip2_with_axes(self.max_size, |min, max, axis| {
                let min = f64::from(min);
                let max = f64::from(max) + 1.0 - f64::EPSILON;
                let mode = min + (max - min) / 2.0;
                Triangular::new(min, max, mode)
                    .map_err(|error| MapGenError::SizeDist(error, axis))
            })
            .transpose()?;

        let top_left = top_left_dist.as_ref().map(|dist| rng.sample(dist));
        let size = size_dist.as_ref().map(|dist| rng.sample(dist));
        let rect = thedes_geometry::Rect { top_left, size };
        let rect = rect.map(|coord| coord as Coord);

        let map = Map::new(rect);
        Ok(map)
    }
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    map_config: MapConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl GameConfig {
    pub fn new() -> Self {
        Self { map_config: MapConfig::new() }
    }

    pub fn with_map(self, map_config: MapConfig) -> Self {
        Self { map_config, ..self }
    }

    pub fn gen(
        self,
        rng: &mut PickedReproducibleRng,
    ) -> Result<Game, GameGenError> {
        let map = self.map_config.generate(rng)?;
        let player_head_dist = map
            .rect()
            .size
            .map_with_axes(|coord, axis| {
                let min = 2.0;
                let max = f64::from(coord) - 2.0 - f64::EPSILON;
                let mode = min + (max - min) / 2.0;
                Triangular::new(min, max, mode)
                    .map_err(|error| GameGenError::PlayerHeadDist(error, axis))
            })
            .transpose()?;
        let player_head_offset =
            player_head_dist.as_ref().map(|dist| rng.sample(dist) as Coord);
        let player_head = map.rect().top_left + player_head_offset;
        let player_facing_index = rng.gen_range(0 .. Direction::ALL.len());
        let player_facing = Direction::ALL[player_facing_index];
        let player = Player::new(player_head, player_facing);
        Ok(Game::new(map, player))
    }
}
