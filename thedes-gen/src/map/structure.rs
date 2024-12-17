use std::convert::Infallible;

use num::rational::Ratio;
use thedes_domain::geometry::{Coord, CoordPair};
use thedes_tui::{component::task::{TaskReset, TaskTick}, geometry::Coord};
use thiserror::Error;

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

pub struct GeneratorTickArgs {}

#[derive(Debug, Clone)]
struct GeneratorResources {
    house_count: Coord,
}

#[derive(Debug, Clone)]
enum GeneratorState {
    Init,
    GenerateHouseCount,
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
            resources: GeneratorResources {
                house_count: 0,
            },
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

impl TaskTick<GeneratorTickArgs> for Generator {
    type Error = Infallible;
    type Output = ();

    fn on_tick(
        &mut self,
        tick: &mut thedes_tui::Tick,
        args: GeneratorTickArgs,
    ) -> Result<Option<Self::Output>, Self::Error> {
        Ok(())
    }
}
