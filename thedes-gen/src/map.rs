use std::{convert::Infallible, mem};

use layer::matter::GroundLayer;
use rand::Rng;
use rand_distr::{Triangular, TriangularError};
use thedes_domain::{
    geometry::{Coord, CoordPair, Rect},
    map::{self, Map},
    matter::Ground,
};
use thedes_geometry::axis::Axis;
use thedes_tui::component::task::{ProgressMetric, Task};
use thiserror::Error;

use self::layer::matter::GroundLayerError;

use super::{matter::GroundDist, random::PickedReproducibleRng};

pub mod layer;

#[derive(Debug, Error)]
pub enum InvalidConfig {
    #[error("Map rectangle {given_rect} has overflowing bottom right point")]
    BottomRightOverflow { given_rect: Rect },
    #[error("Minimum map top left {min} cannot be greater than maximum {max}")]
    TopLeftBoundOrder { min: CoordPair, max: CoordPair },
    #[error("Minimum map size {min} cannot be greater than maximum {max}")]
    SizeBoundOrder { min: CoordPair, max: CoordPair },
}

#[derive(Debug, Error)]
pub enum GenError {
    #[error("Error creating random distribution for map top left's axis {1}")]
    TopLeftDist(#[source] TriangularError, Axis),
    #[error("Error creating random distribution for map size's axis {1}")]
    SizeDist(#[source] TriangularError, Axis),
    #[error("Error generating map ground layer")]
    GroundLayer(
        #[source]
        #[from]
        layer::region::GenError<GroundLayerError>,
    ),
    #[error("Failed to create a map")]
    Creation(
        #[source]
        #[from]
        map::CreationError,
    ),
}

#[derive(Debug, Clone)]
pub struct Config {
    min_top_left: CoordPair,
    max_top_left: CoordPair,
    min_size: CoordPair,
    max_size: CoordPair,
    ground_layer_config: layer::region::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            min_top_left: CoordPair { y: 0, x: 0 },
            max_top_left: CoordPair { y: 10_000, x: 10_000 },
            min_size: CoordPair { y: 950, x: 950 },
            max_size: CoordPair { y: 1050, x: 1050 },
            ground_layer_config: layer::region::Config::new(),
        }
    }

    pub fn with_min_top_left(
        self,
        min_top_left: CoordPair,
    ) -> Result<Self, InvalidConfig> {
        if min_top_left.zip2(self.max_top_left).any(|(min, max)| min > max) {
            Err(InvalidConfig::TopLeftBoundOrder {
                min: min_top_left,
                max: self.max_top_left,
            })?;
        }
        let rect = Rect { top_left: min_top_left, size: self.max_size };
        if rect.checked_bottom_right().is_none() {
            Err(InvalidConfig::BottomRightOverflow { given_rect: rect })?
        }
        Ok(Self { min_top_left, ..self })
    }

    pub fn with_max_top_left(
        self,
        max_top_left: CoordPair,
    ) -> Result<Self, InvalidConfig> {
        if self.min_top_left.zip2(max_top_left).any(|(min, max)| min > max) {
            Err(InvalidConfig::TopLeftBoundOrder {
                min: self.min_top_left,
                max: max_top_left,
            })?;
        }
        let rect = Rect { top_left: max_top_left, size: self.max_size };
        if rect.checked_bottom_right().is_none() {
            Err(InvalidConfig::BottomRightOverflow { given_rect: rect })?
        }
        Ok(Self { max_top_left, ..self })
    }

    pub fn with_min_size(
        self,
        min_size: CoordPair,
    ) -> Result<Self, InvalidConfig> {
        if min_size.zip2(self.max_size).any(|(min, max)| min > max) {
            Err(InvalidConfig::SizeBoundOrder {
                min: min_size,
                max: self.max_size,
            })?;
        }
        let rect = Rect { top_left: self.max_top_left, size: min_size };
        if rect.checked_bottom_right().is_none() {
            Err(InvalidConfig::BottomRightOverflow { given_rect: rect })?
        }
        Ok(Self { min_size, ..self })
    }

    pub fn with_max_size(
        self,
        max_size: CoordPair,
    ) -> Result<Self, InvalidConfig> {
        if self.min_size.zip2(max_size).any(|(min, max)| min > max) {
            Err(InvalidConfig::SizeBoundOrder {
                min: self.min_size,
                max: max_size,
            })?;
        }
        let rect = Rect { top_left: self.max_top_left, size: max_size };
        if rect.checked_bottom_right().is_none() {
            Err(InvalidConfig::BottomRightOverflow { given_rect: rect })?
        }
        Ok(Self { max_size, ..self })
    }

    pub fn with_ground_layer(self, config: layer::region::Config) -> Self {
        Self { ground_layer_config: config, ..self }
    }

    pub fn finish(self) -> Generator {
        let ground_layer_gen =
            self.ground_layer_config.clone().finish(GroundDist::default());
        Generator {
            config: self,
            ground_layer_gen,
            state: GeneratorState::GeneratingRect,
        }
    }
}

#[derive(Debug, Clone)]
enum GeneratorState {
    GeneratingRect,
    GeneratingGroundLayer(Map),
    Done,
}

#[derive(Debug, Clone)]
pub struct Generator {
    config: Config,
    ground_layer_gen: layer::region::Generator<GroundLayer, Ground, GroundDist>,
    state: GeneratorState,
}

impl<'a> Task<'a> for Generator {
    type ResetArgs = Config;
    type ResetOutput = ();
    type ResetError = Infallible;
    type TickArgs = PickedReproducibleRng;
    type TickOutput = Map;
    type TickError = GenError;

    #[inline(always)]
    fn progress_goal(&self) -> ProgressMetric {
        1 + self.ground_layer_gen.progress_goal()
    }

    #[inline(always)]
    fn progress_status(&self) -> ProgressMetric {
        match &self.state {
            GeneratorState::GeneratingRect => 0,
            GeneratorState::GeneratingGroundLayer(_) => {
                1 + self.ground_layer_gen.progress_status()
            },
            GeneratorState::Done => self.progress_goal(),
        }
    }

    fn reset(
        &mut self,
        config: Self::ResetArgs,
    ) -> Result<Self::ResetOutput, Self::ResetError> {
        self.state = GeneratorState::GeneratingRect;
        self.ground_layer_gen.reset(config.ground_layer_config.clone())?;
        self.config = config;
        Ok(())
    }

    #[inline(always)]
    fn on_tick(
        &mut self,
        tick: &mut thedes_tui::Tick,
        rng: &mut Self::TickArgs,
    ) -> Result<Option<Self::TickOutput>, Self::TickError> {
        loop {
            match mem::replace(&mut self.state, GeneratorState::GeneratingRect)
            {
                GeneratorState::Done => {},
                GeneratorState::GeneratingRect => {
                    let top_left_dist = self
                        .config
                        .min_top_left
                        .zip2_with_axes(
                            self.config.max_top_left,
                            |min, max, axis| {
                                let min = f64::from(min);
                                let max = f64::from(max) + 1.0 - f64::EPSILON;
                                let mode = min + (max - min) / 2.0;
                                Triangular::new(min, max, mode).map_err(
                                    |error| GenError::TopLeftDist(error, axis),
                                )
                            },
                        )
                        .transpose()?;
                    let size_dist = self
                        .config
                        .min_size
                        .zip2_with_axes(
                            self.config.max_size,
                            |min, max, axis| {
                                let min = f64::from(min);
                                let max = f64::from(max) + 1.0 - f64::EPSILON;
                                let mode = min + (max - min) / 2.0;
                                Triangular::new(min, max, mode).map_err(
                                    |error| GenError::SizeDist(error, axis),
                                )
                            },
                        )
                        .transpose()?;

                    let top_left =
                        top_left_dist.as_ref().map(|dist| rng.sample(dist));
                    let size = size_dist.as_ref().map(|dist| rng.sample(dist));
                    let rect = thedes_geometry::Rect { top_left, size };
                    let rect = rect.map(|coord| coord as Coord);

                    let map = Map::new(rect)?;
                    self.state = GeneratorState::GeneratingGroundLayer(map)
                },
                GeneratorState::GeneratingGroundLayer(mut map) => {
                    if self
                        .ground_layer_gen
                        .on_tick(tick, &mut (&mut map, &GroundLayer, rng))?
                        .is_some()
                    {
                        self.state = GeneratorState::Done;
                        break Ok(Some(map));
                    }
                    self.state = GeneratorState::GeneratingGroundLayer(map);
                    break Ok(None);
                },
            }
        }
    }
}
