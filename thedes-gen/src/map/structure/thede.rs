use std::{collections::VecDeque, mem};

use num::rational::Ratio;
use rand::Rng;
use thedes_domain::{
    geometry::{Coord, CoordPair},
    map::Map,
};
use thedes_geometry::axis::{
    Axis,
    Diagonal,
    DiagonalMap,
    Direction,
    DirectionMap,
};
use thedes_tui::{
    component::task::{TaskReset, TaskTick},
    Tick,
};
use thiserror::Error;

use crate::{
    map::layer::thede::{InitialLand, InitialLandsCollection},
    random::PickedReproducibleRng,
};

use super::graph::SmallGraph;

#[derive(Debug, Error)]
pub enum GenError {}

#[derive(Debug, Error)]
pub enum ConfigError {
    /*
    #[error(
        "Minimum node-area ratio {0} is too low (must be greater than {})",
        Config::NON_INCL_NODE_AREA_LOW
    )]
    MinNodeAreaRatioTooLow(Ratio<Coord>),
    #[error(
        "Minimum node-area ratio {0} is too high (must be lower than {})",
        Config::NON_INCL_NODE_AREA_HIGH
    )]
    MinNodeAreaRatioTooHigh(Ratio<Coord>),
    #[error(
        "Maximum node-area ratio {0} is too low (must be greater than {})",
        Config::NON_INCL_NODE_AREA_LOW
    )]
    MaxNodeAreaRatioTooLow(Ratio<Coord>),
    #[error(
        "Maximum node-area ratio {0} is too high (must be lower than {})",
        Config::NON_INCL_NODE_AREA_HIGH
    )]
    MaxNodeAreaRatioTooHigh(Ratio<Coord>),
    #[error(
        "Candidate mininum ({min}) and maximum ({max}) node-area ratio are \
         out of order"
    )]
    NodeAreaRatioOrdering { min: Ratio<Coord>, max: Ratio<Coord> },
    #[error(
        "Minimum edge-node ratio {0} is too low (must be at least {})",
        Config::INCL_EDGE_NODE_LOW
    )]
    MinEdgeNodeRatioTooLow(Ratio<Coord>),
    #[error(
        "Minimum edge-node ratio {0} is too high (must be at most {})",
        Config::INCL_EDGE_NODE_HIGH
    )]
    MinEdgeNodeRatioTooHigh(Ratio<Coord>),
    #[error(
        "Maximum edge-node ratio {0} is too low (must be at least {})",
        Config::INCL_EDGE_NODE_LOW
    )]
    MaxEdgeNodeRatioTooLow(Ratio<Coord>),
    #[error(
        "Maximum edge-node ratio {0} is too high (must be at most {})",
        Config::INCL_EDGE_NODE_HIGH
    )]
    MaxEdgeNodeRatioTooHigh(Ratio<Coord>),
    #[error(
        "Candidate mininum ({min}) and maximum ({max}) edge-node ratio are \
         out of order"
    )]
    EdgeNodeRatioOrdering { min: Ratio<Coord>, max: Ratio<Coord> },
    */
}

#[derive(Debug, Clone)]
pub struct Config {
    min_house_size: CoordPair,
    max_house_size: CoordPair,
    min_houses_ratio: Ratio<Coord>,
    max_houses_ratio: Ratio<Coord>,
    /*
    min_node_area_ratio: Ratio<Coord>,
    max_node_area_ratio: Ratio<Coord>,
    min_edge_node_ratio: Ratio<Coord>,
    max_edge_node_ratio: Ratio<Coord>,
        */
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_house_size: CoordPair::from_axes(|_| 3),
            max_house_size: CoordPair::from_axes(|_| 10),
            min_houses_ratio: Ratio::new_raw(1, 20),
            max_houses_ratio: Ratio::ONE,
            /*
                min_node_area_ratio: Ratio::new(1, 25),
                max_node_area_ratio: Ratio::new(1, 20),
                min_edge_node_ratio: Ratio::new(1, 1),
                max_edge_node_ratio: Ratio::new(2, 1),
            */
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /*
    pub const NON_INCL_NODE_AREA_LOW: Ratio<Coord> = Ratio::ZERO;
    pub const NON_INCL_NODE_AREA_HIGH: Ratio<Coord> = Ratio::ONE;

    pub const INCL_EDGE_NODE_LOW: Ratio<Coord> = Ratio::ONE;
    pub const INCL_EDGE_NODE_HIGH: Ratio<Coord> = Ratio::new_raw(4, 1);


    pub fn with_min_node_area_ratio(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, ConfigError> {
        if ratio <= Self::NON_INCL_NODE_AREA_LOW {
            Err(ConfigError::MinNodeAreaRatioTooLow(ratio))?
        }
        if ratio >= Self::NON_INCL_NODE_AREA_HIGH {
            Err(ConfigError::MinNodeAreaRatioTooHigh(ratio))?
        }
        if ratio > self.max_node_area_ratio {
            Err(ConfigError::NodeAreaRatioOrdering {
                min: ratio,
                max: self.max_node_area_ratio,
            })?
        }
        Ok(Self { min_node_area_ratio: ratio, ..self })
    }

    pub fn with_max_node_area_ratio(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, ConfigError> {
        if ratio <= Self::NON_INCL_NODE_AREA_LOW {
            Err(ConfigError::MaxNodeAreaRatioTooLow(ratio))?
        }
        if ratio >= Self::NON_INCL_NODE_AREA_HIGH {
            Err(ConfigError::MaxNodeAreaRatioTooHigh(ratio))?
        }
        if ratio < self.min_node_area_ratio {
            Err(ConfigError::NodeAreaRatioOrdering {
                min: self.min_node_area_ratio,
                max: ratio,
            })?
        }
        Ok(Self { max_node_area_ratio: ratio, ..self })
    }

    pub fn with_min_edge_node_ratio(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, ConfigError> {
        if ratio < Self::INCL_EDGE_NODE_LOW {
            Err(ConfigError::MinEdgeNodeRatioTooLow(ratio))?
        }
        if ratio > Self::INCL_EDGE_NODE_HIGH {
            Err(ConfigError::MinEdgeNodeRatioTooHigh(ratio))?
        }
        if ratio > self.max_edge_node_ratio {
            Err(ConfigError::EdgeNodeRatioOrdering {
                min: ratio,
                max: self.max_edge_node_ratio,
            })?
        }
        Ok(Self { min_edge_node_ratio: ratio, ..self })
    }

    pub fn with_max_edge_node_ratio(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, ConfigError> {
        if ratio < Self::INCL_EDGE_NODE_LOW {
            Err(ConfigError::MaxEdgeNodeRatioTooLow(ratio))?
        }
        if ratio > Self::INCL_EDGE_NODE_HIGH {
            Err(ConfigError::MaxEdgeNodeRatioTooHigh(ratio))?
        }
        if ratio < self.min_edge_node_ratio {
            Err(ConfigError::EdgeNodeRatioOrdering {
                min: self.min_edge_node_ratio,
                max: ratio,
            })?
        }
        Ok(Self { max_edge_node_ratio: ratio, ..self })
    }

    pub fn min_node_area_ratio(&self) -> Ratio<Coord> {
        self.min_node_area_ratio
    }

    pub fn max_node_area_ratio(&self) -> Ratio<Coord> {
        self.max_node_area_ratio
    }

    pub fn min_edge_node_ratio(&self) -> Ratio<Coord> {
        self.min_edge_node_ratio
    }

    pub fn max_edge_node_ratio(&self) -> Ratio<Coord> {
        self.max_edge_node_ratio
    }
    */

    pub fn finish(self) -> Generator {
        Generator {
            state: GeneratorState::INITIAL,
            resources: GeneratorResources::new(self),
        }
    }
}

#[derive(Debug)]
pub struct GeneratorResetArgs<'c> {
    pub initial_lands: &'c mut InitialLandsCollection,
    pub config: Config,
}

#[derive(Debug)]
pub struct GeneratorTickArgs<'m, 'r> {
    pub map: &'m mut Map,
    pub rng: &'r mut PickedReproducibleRng,
}

#[derive(Debug, Clone)]
enum GeneratorState {
    InitingCenter { thede_index: usize },
    GeneratingHouseCount { thede_index: usize },
    Expanding { thede_index: usize, land: InitialLand, houses_left: Coord },
    Done,
}

impl GeneratorState {
    pub const INITIAL: Self = Self::InitingCenter { thede_index: 0 };
}

#[derive(Debug, Clone)]
struct GeneratorResources {
    config: Config,
    frontiers: VecDeque<CoordPair>,
    initial_lands: Vec<InitialLand>,
    current_graph: SmallGraph<()>,
}

impl GeneratorResources {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            frontiers: VecDeque::new(),
            initial_lands: Vec::new(),
            current_graph: SmallGraph::new(),
        }
    }

    pub fn transition(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs,
        state: GeneratorState,
    ) -> Result<GeneratorState, GenError> {
        match state {
            GeneratorState::InitingCenter { thede_index } => {
                self.initing_center(tick, args, thede_index)
            },
            GeneratorState::Expanding { thede_index, houses_left, land } => {
                self.expanding(tick, args, thede_index, houses_left, land)
            },
            GeneratorState::Done => self.done(tick, args),
        }
    }

    pub fn initing_center(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs,
        thede_index: usize,
    ) -> Result<GeneratorState, GenError> {
        if let Some(land) = self.initial_lands.get(thede_index).copied() {
            self.frontiers.clear();
            self.frontiers.push_back(land.location);
            self.current_graph.clear();
            self.current_graph.insert_node(land.location, ());

            let max_house_area = self
                .config
                .max_house_size
                .y
                .saturating_mul(self.config.max_house_size.x);
            let theoretical_house_limit = land.area / max_house_area;
            let min_houses_adjusted =
                self.config.min_houses_ratio * theoretical_house_limit;
            let min_houses = min_houses_adjusted.round().to_integer().min(1);
            let max_houses_adjusted =
                self.config.max_houses_ratio * theoretical_house_limit;
            let max_houses =
                max_houses_adjusted.round().to_integer().max(min_houses);
            let house_count = args.rng.gen_range(min_houses ..= max_houses);

            Ok(GeneratorState::Expanding {
                thede_index,
                houses_left: house_count,
                land,
            })
        } else {
            Ok(GeneratorState::Done)
        }
    }

    pub fn expanding(
        &mut self,
        _tick: &mut Tick,
        args: GeneratorTickArgs,
        thede_index: usize,
        mut houses_left: Coord,
        land: InitialLand,
    ) -> Result<GeneratorState, GenError> {
        if let Some(node) = self.frontiers.pop_front() {
            let mut max_houses_towards = DiagonalMap::default();
            let direction_max = DirectionMap::default();
            for direction in Direction::ALL {}
        }
        /*
        houses_left -= 1;
        if houses_left == 0 {
            todo!()
        } else {
            todo!()
        }
        */
    }

    pub fn done(
        &mut self,
        _tick: &mut Tick,
        _args: GeneratorTickArgs,
    ) -> Result<GeneratorState, GenError> {
        Ok(GeneratorState::Done)
    }
}

#[derive(Debug, Clone)]
pub struct Generator {
    state: GeneratorState,
    resources: GeneratorResources,
}

impl<'c> TaskReset<GeneratorResetArgs<'c>> for Generator {
    type Output = ();
    type Error = GenError;

    fn reset(
        &mut self,
        args: GeneratorResetArgs,
    ) -> Result<Self::Output, Self::Error> {
        self.state = GeneratorState::INITIAL;
        self.resources.config = args.config;
        self.resources.frontiers.clear();
        self.resources.initial_lands.clear();
        self.resources.initial_lands.extend(args.initial_lands.drain());
        self.resources.current_graph.clear();
        Ok(())
    }
}

impl<'m, 'r> TaskTick<GeneratorTickArgs<'m, 'r>> for Generator {
    type Error = GenError;
    type Output = ();

    fn on_tick(
        &mut self,
        tick: &mut Tick,
        args: GeneratorTickArgs<'m, 'r>,
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
