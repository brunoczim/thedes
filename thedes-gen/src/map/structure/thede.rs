use std::{convert::Infallible, mem};

use num::rational::Ratio;
use thedes_domain::{
    geometry::{Coord, CoordPair},
    map::Map,
    thede,
};
use thedes_tui::{
    component::task::{ProgressMetric, TaskReset, TaskTick},
    Tick,
};
use thiserror::Error;

use crate::{
    map::layer::thede::{InitialLand, InitialLandsCollector},
    random::PickedReproducibleRng,
};

#[derive(Debug, Error)]
pub enum GenError {}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(
        "House count ratio {} is too small, limit is {}",
        .0,
        Config::MIN_HOUSE_COUNT_RATIO,
    )]
    HouseRatioTooSmall(Ratio<Coord>),
    #[error(
        "House count ratio {} is too big, limit is {}",
        .0,
        Config::MAX_HOUSE_COUNT_RATIO,
    )]
    HouseRatioTooBig(Ratio<Coord>),
    #[error(
        "House size {} is too small, limit is {}",
        .0,
        Config::MIN_HOUSE_SIZE_LIMIT,
    )]
    HouseSizeTooSmall(CoordPair),
    #[error(
        "House size {} is too big, limit is {}",
        .0,
        Config::MAX_HOUSE_SIZE_LIMIT,
    )]
    HouseSizeTooBig(CoordPair),
}

#[derive(Debug, Clone)]
pub struct Config {
    min_house_count: Ratio<Coord>,
    max_house_count: Ratio<Coord>,
    min_house_size: CoordPair,
    max_house_size: CoordPair,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_house_count: Ratio::new(1, 20),
            max_house_count: Ratio::new(1, 15),
            min_house_size: Self::MIN_HOUSE_SIZE_LIMIT,
            max_house_size: CoordPair::from_axes(|_| 20),
        }
    }
}

impl Config {
    pub const MIN_HOUSE_COUNT_RATIO: Ratio<Coord> = Ratio::new_raw(1, 100);
    pub const MAX_HOUSE_COUNT_RATIO: Ratio<Coord> = Ratio::new_raw(1, 1);
    pub const MIN_HOUSE_SIZE_LIMIT: CoordPair = CoordPair { y: 5, x: 5 };
    pub const MAX_HOUSE_SIZE_LIMIT: CoordPair = CoordPair { y: 50, x: 50 };

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_house_count(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, ConfigError> {
        if ratio < Self::MIN_HOUSE_COUNT_RATIO {
            Err(ConfigError::HouseRatioTooSmall(ratio))?
        }
        if ratio > Self::MAX_HOUSE_COUNT_RATIO {
            Err(ConfigError::HouseRatioTooBig(ratio))?
        }
        Ok(Self { min_house_count: ratio, ..self })
    }

    pub fn with_max_house_count(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, ConfigError> {
        if ratio < Self::MIN_HOUSE_COUNT_RATIO {
            Err(ConfigError::HouseRatioTooSmall(ratio))?
        }
        if ratio > Self::MAX_HOUSE_COUNT_RATIO {
            Err(ConfigError::HouseRatioTooBig(ratio))?
        }
        Ok(Self { max_house_count: ratio, ..self })
    }

    pub fn with_min_house_size(
        self,
        size: CoordPair,
    ) -> Result<Self, ConfigError> {
        if size < Self::MIN_HOUSE_SIZE_LIMIT {
            Err(ConfigError::HouseSizeTooSmall(size))?;
        }
        if size > Self::MAX_HOUSE_SIZE_LIMIT {
            Err(ConfigError::HouseSizeTooBig(size))?;
        }
        Ok(Self { min_house_size: size, ..self })
    }

    pub fn with_max_house_size(
        self,
        size: CoordPair,
    ) -> Result<Self, ConfigError> {
        if size < Self::MIN_HOUSE_SIZE_LIMIT {
            Err(ConfigError::HouseSizeTooSmall(size))?;
        }
        if size > Self::MAX_HOUSE_SIZE_LIMIT {
            Err(ConfigError::HouseSizeTooBig(size))?;
        }
        Ok(Self { max_house_size: size, ..self })
    }

    pub fn min_house_count(&self) -> Ratio<Coord> {
        self.min_house_count
    }

    pub fn max_house_count(&self) -> Ratio<Coord> {
        self.max_house_count
    }

    pub fn min_house_size(&self) -> CoordPair {
        self.min_house_size
    }

    pub fn max_house_size(&self) -> CoordPair {
        self.max_house_size
    }
}

#[derive(Debug)]
pub struct GeneratorTickArgs<'m, 'r, 'c> {
    pub map: &'m mut Map,
    pub rng: &'r mut PickedReproducibleRng,
    pub land_collector: &'c mut InitialLandsCollector,
}

#[derive(Debug, Clone)]
struct GeneratorResources {
    lands: Vec<InitialLand>,
    current_thede_land: usize,
    house_count: Coord,
    progress_goal: ProgressMetric,
    current_progress: ProgressMetric,
}

impl GeneratorResources {
    fn transition(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs,
        state: GeneratorState,
    ) -> Result<GeneratorState, GenError> {
        match state {
            GeneratorState::Init => self.init(tick, args),
            GeneratorState::InitThede => self.init_thede(tick, args),
            GeneratorState::Done => self.done(tick, args),
        }
    }

    fn done(
        &mut self,
        _tick: &mut Tick,
        _args: GeneratorTickArgs,
    ) -> Result<GeneratorState, GenError> {
        self.current_progress = self.progress_goal;
        Ok(GeneratorState::Done)
    }

    fn init(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs,
    ) -> Result<GeneratorState, GenError> {
        self.lands.extend(args.land_collector.drain());
        self.init_thede(tick, args)
    }

    fn init_thede(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs,
    ) -> Result<GeneratorState, GenError> {
    }
}

#[derive(Debug, Clone)]
enum GeneratorState {
    Init,
    InitThede,
    Done,
}

impl GeneratorState {
    pub const INITIAL: Self = Self::Init;
}

#[derive(Debug, Clone)]
pub struct Generator {
    state: GeneratorState,
    resources: GeneratorResources,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            state: GeneratorState::INITIAL,
            resources: GeneratorResources { house_count: 0 },
        }
    }
}

impl TaskReset<()> for Generator {
    type Output = ();
    type Error = Infallible;

    fn reset(&mut self, _args: ()) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

impl<'m, 'r, 'c> TaskTick<GeneratorTickArgs<'m, 'r, 'c>> for Generator {
    type Error = GenError;
    type Output = ();

    fn on_tick(
        &mut self,
        tick: &mut thedes_tui::Tick,
        args: GeneratorTickArgs,
    ) -> Result<Option<Self::Output>, Self::Error> {
        let current_state =
            mem::replace(&mut self.state, GeneratorState::INITIAL);
        self.state = self.resources.transition(tick, args, current_state)?;
        match &self.state {
            GeneratorState::Done => Ok(Some(())),
            _ => Ok(None),
        }
    }
}
