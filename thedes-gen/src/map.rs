use std::convert::Infallible;

use layer::matter::{
    BiomeLayer,
    BiomeLayerError,
    GroundDistrError,
    GroundLayer,
    GroundLayerDistr,
    GroundLayerError,
};
use rand::Rng;
use rand_distr::{Triangular, TriangularError};
use thedes_domain::{
    geometry::{Coord, CoordPair, Rect},
    map::{self, Map},
};
use thedes_geometry::orientation::Axis;
use thiserror::Error;

use crate::{matter::BiomeDistr, progress, random::PickedReproducibleRng};

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
pub enum InitError {
    #[error("Failed to initialize biome layer generator")]
    Biome(#[source] layer::region::InitError),
    #[error("Error creating random distribution for map top left's axis {1}")]
    TopLeftDistr(#[source] TriangularError, Axis),
    #[error("Error creating random distribution for map size's axis {1}")]
    SizeDistr(#[source] TriangularError, Axis),
    #[error("Failed to create a map")]
    Creation(
        #[source]
        #[from]
        map::InitError,
    ),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error generating map bioome layer")]
    BiomeLayer(
        #[source]
        #[from]
        layer::region::Error<BiomeLayerError, Infallible, Infallible>,
    ),
    #[error("Error generating map ground layer")]
    GroundLayer(
        #[source]
        #[from]
        layer::pointwise::Error<GroundLayerError, GroundDistrError>,
    ),
}

#[derive(Debug, Clone)]
pub struct Config {
    min_top_left: CoordPair,
    max_top_left: CoordPair,
    min_size: CoordPair,
    max_size: CoordPair,
    biome_distr: BiomeDistr,
    biome_layer_config: layer::region::Config,
    ground_layer_distr: GroundLayerDistr,
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
            biome_distr: BiomeDistr::default(),
            biome_layer_config: layer::region::Config::new(),
            ground_layer_distr: GroundLayerDistr::default(),
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

    pub fn with_biome_layer(self, config: layer::region::Config) -> Self {
        Self { biome_layer_config: config, ..self }
    }

    pub fn with_biome_distr(self, distr: BiomeDistr) -> Self {
        Self { biome_distr: distr, ..self }
    }

    pub fn with_ground_layer_distr(self, distr: GroundLayerDistr) -> Self {
        Self { ground_layer_distr: distr, ..self }
    }

    pub fn finish(
        self,
        rng: &mut PickedReproducibleRng,
    ) -> Result<Generator, InitError> {
        let top_left_distr = self
            .min_top_left
            .zip2_with_axes(self.max_top_left, |min, max, axis| {
                let min = f64::from(min);
                let max = f64::from(max) + 1.0 - f64::EPSILON;
                let mode = min + (max - min) / 2.0;
                Triangular::new(min, max, mode)
                    .map_err(|error| InitError::TopLeftDistr(error, axis))
            })
            .transpose()?;
        let size_distr = self
            .min_size
            .zip2_with_axes(self.max_size, |min, max, axis| {
                let min = f64::from(min);
                let max = f64::from(max) + 1.0 - f64::EPSILON;
                let mode = min + (max - min) / 2.0;
                Triangular::new(min, max, mode)
                    .map_err(|error| InitError::SizeDistr(error, axis))
            })
            .transpose()?;

        let top_left = top_left_distr.as_ref().map(|distr| rng.sample(distr));
        let size = size_distr.as_ref().map(|distr| rng.sample(distr));
        let rect = thedes_geometry::Rect { top_left, size };
        let rect = rect.map(|coord| coord as Coord);

        let map = Map::new(rect)?;

        let biome_layer_gen = self
            .biome_layer_config
            .clone()
            .finish(&map, rng)
            .map_err(InitError::Biome)?;
        let ground_layer_gen = layer::pointwise::Generator::new();

        Ok(Generator { config: self, map, biome_layer_gen, ground_layer_gen })
    }
}

#[derive(Debug)]
pub struct Generator {
    config: Config,
    map: Map,
    biome_layer_gen: layer::region::Generator,
    ground_layer_gen: layer::pointwise::Generator,
}

impl Generator {
    pub fn progress_goal(&self) -> usize {
        self.biome_layer_gen.progress_goal(&self.map)
            + self.ground_layer_gen.progress_goal(&self.map)
    }

    pub async fn execute(
        mut self,
        rng: &mut PickedReproducibleRng,
        progress_logger: progress::Logger,
    ) -> Result<Map, Error> {
        progress_logger.set_status("generating biome layer");
        self.biome_layer_gen
            .execute(
                &BiomeLayer,
                &mut self.config.biome_distr,
                &mut self.map,
                rng,
                &mut layer::region::NopCollector,
                progress_logger.nest(),
            )
            .await?;

        progress_logger.set_status("generating ground layer");
        self.ground_layer_gen
            .execute(
                &GroundLayer,
                &mut self.config.ground_layer_distr,
                &mut self.map,
                rng,
                progress_logger.nest(),
            )
            .await?;

        progress_logger.set_status("done");

        Ok(self.map)
    }
}
