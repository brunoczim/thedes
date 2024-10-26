use std::{convert::Infallible, mem};

use layer::{
    block::{BlockLayer, BlockLayerDistr},
    matter::{
        BiomeLayer,
        BiomeLayerError,
        GroundDistrError,
        GroundLayer,
        GroundLayerDistr,
    },
    thede::{ThedeLayer, ThedeLayerError},
};
use rand::Rng;
use rand_distr::{Triangular, TriangularError};
use thedes_domain::{
    geometry::{Coord, CoordPair, Rect},
    map::{self, Map},
    matter::Biome,
    thede,
};
use thedes_geometry::axis::Axis;
use thedes_tui::{
    component::task::{ProgressMetric, TaskProgress, TaskReset, TaskTick},
    Tick,
};
use thiserror::Error;

use crate::{
    matter::BiomeDistr,
    thede::{ThedeDistr, ThedeDistrError},
};

use self::layer::matter::GroundLayerError;

use super::random::PickedReproducibleRng;

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
    TopLeftDistr(#[source] TriangularError, Axis),
    #[error("Error creating random distribution for map size's axis {1}")]
    SizeDistr(#[source] TriangularError, Axis),
    #[error("Error generating map bioome layer")]
    BiomeLayer(
        #[source]
        #[from]
        layer::region::GenError<BiomeLayerError, Infallible>,
    ),
    #[error("Error generating map ground layer")]
    GroundLayer(
        #[source]
        #[from]
        layer::pointwise::GenError<GroundLayerError, GroundDistrError>,
    ),
    #[error("Error generating map block layer")]
    BlockLayer(
        #[source]
        #[from]
        layer::pointwise::GenError<
            layer::block::LayerError,
            layer::block::DistrError,
        >,
    ),
    #[error("Error generating map thede layer")]
    ThedeLayer(
        #[source]
        #[from]
        layer::region::GenError<ThedeLayerError, ThedeDistrError>,
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
    biome_distr: BiomeDistr,
    biome_layer_config: layer::region::Config,
    ground_layer_distr: GroundLayerDistr,
    block_layer_distr: BlockLayerDistr,
    thede_distr: ThedeDistr,
    thede_layer_config: layer::region::Config,
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
            block_layer_distr: BlockLayerDistr::default(),
            thede_distr: ThedeDistr::default(),
            thede_layer_config: layer::region::Config::new(),
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

    pub fn with_block_layer_distr(self, distr: BlockLayerDistr) -> Self {
        Self { block_layer_distr: distr, ..self }
    }

    pub fn with_thede_layer(self, config: layer::region::Config) -> Self {
        Self { thede_layer_config: config, ..self }
    }

    pub fn with_thede_distr(self, distr: ThedeDistr) -> Self {
        Self { thede_distr: distr, ..self }
    }

    pub fn finish(self) -> Generator {
        let biome_layer_gen = self.biome_layer_config.clone().finish();
        let ground_layer_gen = layer::pointwise::Generator::new();
        let block_layer_gen = layer::pointwise::Generator::new();
        let thede_layer_gen = self.thede_layer_config.clone().finish();
        Generator {
            resources: GeneratorResources {
                config: self,
                biome_layer_gen,
                ground_layer_gen,
                block_layer_gen,
                thede_layer_gen,
            },
            state: GeneratorState::INITIAL,
        }
    }
}

#[derive(Debug, Clone)]
enum GeneratorState {
    GeneratingRect,
    GeneratingBiomeLayer(Map),
    GeneratingGroundLayer(Map),
    GeneratingBlockLayer(Map),
    GeneratingThedeLayer(Map),
    Done(Option<Map>),
}

impl GeneratorState {
    pub const INITIAL: Self = GeneratorState::GeneratingRect;
}

#[derive(Debug, Clone)]
struct GeneratorResources {
    config: Config,
    biome_layer_gen: layer::region::Generator<Biome>,
    ground_layer_gen: layer::pointwise::Generator,
    block_layer_gen: layer::pointwise::Generator,
    thede_layer_gen: layer::region::Generator<Option<thede::Id>>,
}

impl GeneratorResources {
    fn transition(
        &mut self,
        tick: &mut Tick,
        rng: &mut PickedReproducibleRng,
        state: GeneratorState,
    ) -> Result<GeneratorState, GenError> {
        match state {
            GeneratorState::Done(_) => self.done(tick, rng),
            GeneratorState::GeneratingRect => self.generating_rect(tick, rng),
            GeneratorState::GeneratingBiomeLayer(map) => {
                self.generating_biome_layer(tick, rng, map)
            },
            GeneratorState::GeneratingGroundLayer(map) => {
                self.generating_ground_layer(tick, rng, map)
            },
            GeneratorState::GeneratingBlockLayer(map) => {
                self.generating_block_layer(tick, rng, map)
            },
            GeneratorState::GeneratingThedeLayer(map) => {
                self.generating_thede_layer(tick, rng, map)
            },
        }
    }

    fn done(
        &mut self,
        _tick: &mut Tick,
        _rng: &mut PickedReproducibleRng,
    ) -> Result<GeneratorState, GenError> {
        Ok(GeneratorState::Done(None))
    }

    fn generating_rect(
        &mut self,
        _tick: &mut Tick,
        rng: &mut PickedReproducibleRng,
    ) -> Result<GeneratorState, GenError> {
        let top_left_distr = self
            .config
            .min_top_left
            .zip2_with_axes(self.config.max_top_left, |min, max, axis| {
                let min = f64::from(min);
                let max = f64::from(max) + 1.0 - f64::EPSILON;
                let mode = min + (max - min) / 2.0;
                Triangular::new(min, max, mode)
                    .map_err(|error| GenError::TopLeftDistr(error, axis))
            })
            .transpose()?;
        let size_distr = self
            .config
            .min_size
            .zip2_with_axes(self.config.max_size, |min, max, axis| {
                let min = f64::from(min);
                let max = f64::from(max) + 1.0 - f64::EPSILON;
                let mode = min + (max - min) / 2.0;
                Triangular::new(min, max, mode)
                    .map_err(|error| GenError::SizeDistr(error, axis))
            })
            .transpose()?;

        let top_left = top_left_distr.as_ref().map(|distr| rng.sample(distr));
        let size = size_distr.as_ref().map(|distr| rng.sample(distr));
        let rect = thedes_geometry::Rect { top_left, size };
        let rect = rect.map(|coord| coord as Coord);

        let map = Map::new(rect)?;
        self.block_layer_gen.fit_progress_goal(map.rect());
        Ok(GeneratorState::GeneratingBiomeLayer(map))
    }

    fn generating_biome_layer(
        &mut self,
        tick: &mut Tick,
        rng: &mut PickedReproducibleRng,
        mut map: Map,
    ) -> Result<GeneratorState, GenError> {
        if self
            .biome_layer_gen
            .on_tick(
                tick,
                layer::region::GeneratorTickArgs {
                    map: &mut map,
                    layer: &BiomeLayer,
                    rng,
                    data_distr: &mut self.config.biome_distr,
                },
            )?
            .is_some()
        {
            Ok(GeneratorState::GeneratingGroundLayer(map))
        } else {
            Ok(GeneratorState::GeneratingBiomeLayer(map))
        }
    }

    fn generating_ground_layer(
        &mut self,
        tick: &mut Tick,
        rng: &mut PickedReproducibleRng,
        mut map: Map,
    ) -> Result<GeneratorState, GenError> {
        if self
            .ground_layer_gen
            .on_tick(
                tick,
                layer::pointwise::GeneratorTickArgs {
                    map: &mut map,
                    layer: &GroundLayer,
                    rng,
                    layer_distr: &self.config.ground_layer_distr,
                },
            )?
            .is_some()
        {
            Ok(GeneratorState::GeneratingBlockLayer(map))
        } else {
            Ok(GeneratorState::GeneratingGroundLayer(map))
        }
    }

    fn generating_block_layer(
        &mut self,
        tick: &mut Tick,
        rng: &mut PickedReproducibleRng,
        mut map: Map,
    ) -> Result<GeneratorState, GenError> {
        if self
            .block_layer_gen
            .on_tick(
                tick,
                layer::pointwise::GeneratorTickArgs {
                    map: &mut map,
                    layer: &BlockLayer,
                    rng,
                    layer_distr: &self.config.block_layer_distr,
                },
            )?
            .is_some()
        {
            Ok(GeneratorState::GeneratingThedeLayer(map))
        } else {
            Ok(GeneratorState::GeneratingBlockLayer(map))
        }
    }

    fn generating_thede_layer(
        &mut self,
        tick: &mut Tick,
        rng: &mut PickedReproducibleRng,
        mut map: Map,
    ) -> Result<GeneratorState, GenError> {
        if self
            .thede_layer_gen
            .on_tick(
                tick,
                layer::region::GeneratorTickArgs {
                    map: &mut map,
                    layer: &ThedeLayer,
                    rng,
                    data_distr: &mut self.config.thede_distr,
                },
            )?
            .is_some()
        {
            Ok(GeneratorState::Done(Some(map)))
        } else {
            Ok(GeneratorState::GeneratingThedeLayer(map))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Generator {
    resources: GeneratorResources,
    state: GeneratorState,
}

impl TaskProgress for Generator {
    fn progress_goal(&self) -> ProgressMetric {
        1 + self.resources.biome_layer_gen.progress_goal()
            + self.resources.ground_layer_gen.progress_goal()
            + self.resources.block_layer_gen.progress_goal()
            + self.resources.thede_layer_gen.progress_goal()
    }

    fn current_progress(&self) -> ProgressMetric {
        match &self.state {
            GeneratorState::GeneratingRect => 0,
            GeneratorState::GeneratingBiomeLayer(_) => {
                1 + self.resources.biome_layer_gen.current_progress()
            },
            GeneratorState::GeneratingGroundLayer(_) => {
                1 + self.resources.biome_layer_gen.progress_goal()
                    + self.resources.ground_layer_gen.current_progress()
            },
            GeneratorState::GeneratingBlockLayer(_) => {
                1 + self.resources.biome_layer_gen.progress_goal()
                    + self.resources.ground_layer_gen.progress_goal()
                    + self.resources.block_layer_gen.current_progress()
            },
            GeneratorState::GeneratingThedeLayer(_) => {
                1 + self.resources.biome_layer_gen.progress_goal()
                    + self.resources.ground_layer_gen.progress_goal()
                    + self.resources.block_layer_gen.progress_goal()
                    + self.resources.thede_layer_gen.current_progress()
            },
            GeneratorState::Done(_) => self.progress_goal(),
        }
    }

    fn progress_status(&self) -> String {
        match &self.state {
            GeneratorState::GeneratingRect => "generating rect".to_owned(),
            GeneratorState::GeneratingBiomeLayer(_) => {
                format!(
                    "generating biome layer > {}",
                    self.resources.biome_layer_gen.progress_status()
                )
            },
            GeneratorState::GeneratingGroundLayer(_) => {
                format!(
                    "generating ground layer > {}",
                    self.resources.ground_layer_gen.progress_status()
                )
            },
            GeneratorState::GeneratingBlockLayer(_) => {
                format!(
                    "generating block layer > {}",
                    self.resources.block_layer_gen.progress_status()
                )
            },
            GeneratorState::GeneratingThedeLayer(_) => {
                format!(
                    "generating thede layer > {}",
                    self.resources.thede_layer_gen.progress_status()
                )
            },
            GeneratorState::Done(_) => "done".to_owned(),
        }
    }
}

impl TaskReset<Config> for Generator {
    type Output = ();
    type Error = Infallible;

    fn reset(&mut self, config: Config) -> Result<Self::Output, Self::Error> {
        self.state = GeneratorState::INITIAL;
        self.resources
            .biome_layer_gen
            .reset(config.biome_layer_config.clone())?;
        self.resources.block_layer_gen.reset(())?;
        self.resources.ground_layer_gen.reset(())?;
        self.resources
            .thede_layer_gen
            .reset(self.resources.config.thede_layer_config.clone())?;
        self.resources.config = config;
        Ok(())
    }
}

impl<'a> TaskTick<&'a mut PickedReproducibleRng> for Generator {
    type Output = Map;
    type Error = GenError;

    fn on_tick(
        &mut self,
        tick: &mut thedes_tui::Tick,
        rng: &'a mut PickedReproducibleRng,
    ) -> Result<Option<Self::Output>, Self::Error> {
        let current_state =
            mem::replace(&mut self.state, GeneratorState::INITIAL);
        self.state = self.resources.transition(tick, rng, current_state)?;
        match &mut self.state {
            GeneratorState::Done(output) => Ok(output.take()),
            _ => Ok(None),
        }
    }
}
