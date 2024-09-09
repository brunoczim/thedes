use std::{convert::Infallible, mem};

use rand::Rng;
use rand_distr::{Triangular, TriangularError};
use thedes_domain::{
    game::{self, Game},
    geometry::Coord,
    map::Map,
    player::{self, PlayerPosition},
};
use thedes_geometry::axis::{Axis, Direction};
use thedes_tui::{
    component::task::{ProgressMetric, TaskProgress, TaskReset, TaskTick},
    Tick,
};
use thiserror::Error;

use crate::random::{create_reproducible_rng, Seed};

use super::{map, random::PickedReproducibleRng};

#[derive(Debug, Error)]
pub enum GenError {
    #[error("Error generating map")]
    Map(
        #[from]
        #[source]
        map::GenError,
    ),
    #[error(
        "Error creating random distribution for player's head in axis {1}"
    )]
    PlayerHeadDist(#[source] TriangularError, Axis),
    #[error("Failed to create a game")]
    Creation(
        #[source]
        #[from]
        game::CreationError,
    ),
    #[error("Failed to create a player")]
    CreatePlayer(
        #[source]
        #[from]
        player::CreationError,
    ),
}

#[derive(Debug, Clone)]
pub struct Config {
    map_config: map::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self { map_config: map::Config::new() }
    }

    pub fn with_map(self, map_config: map::Config) -> Self {
        Self { map_config, ..self }
    }

    pub fn finish(self) -> Generator {
        Generator {
            state: GeneratorState::INITIAL,
            resources: GeneratorResources {
                rng: create_reproducible_rng(0),
                map_gen: self.map_config.finish(),
            },
        }
    }
}

#[derive(Debug, Clone)]
enum GeneratorState {
    GeneratingMap,
    GeneratingPlayer(Map),
    Done(Option<Game>),
}

impl GeneratorState {
    pub const INITIAL: Self = Self::GeneratingMap;
}

#[derive(Debug, Clone)]
struct GeneratorResources {
    rng: PickedReproducibleRng,
    map_gen: map::Generator,
}

#[derive(Debug, Clone)]
pub struct GeneratorResetArgs {
    pub seed: Seed,
    pub config: Config,
}

#[derive(Debug, Clone)]
pub struct Generator {
    state: GeneratorState,
    resources: GeneratorResources,
}

impl Generator {
    fn transition(
        &mut self,
        tick: &mut Tick,
        args: &mut (),
        state: GeneratorState,
    ) -> Result<GeneratorState, GenError> {
        match state {
            GeneratorState::Done(_) => self.done(tick, args),
            GeneratorState::GeneratingMap => self.generating_map(tick, args),
            GeneratorState::GeneratingPlayer(map) => {
                self.generating_player(tick, args, map)
            },
        }
    }

    fn done(
        &mut self,
        _tick: &mut Tick,
        _args: &mut (),
    ) -> Result<GeneratorState, GenError> {
        Ok(GeneratorState::Done(None))
    }

    fn generating_map(
        &mut self,
        tick: &mut Tick,
        _args: &mut (),
    ) -> Result<GeneratorState, GenError> {
        match self.resources.map_gen.on_tick(tick, &mut self.resources.rng)? {
            Some(map) => Ok(GeneratorState::GeneratingPlayer(map)),
            None => Ok(GeneratorState::GeneratingMap),
        }
    }

    fn generating_player(
        &mut self,
        _tick: &mut Tick,
        _args: &mut (),
        map: Map,
    ) -> Result<GeneratorState, GenError> {
        let player_head_dist = map
            .rect()
            .size
            .map_with_axes(|coord, axis| {
                let min = 2.0;
                let max = f64::from(coord) - 2.0 - f64::EPSILON;
                let mode = min + (max - min) / 2.0;
                Triangular::new(min, max, mode)
                    .map_err(|error| GenError::PlayerHeadDist(error, axis))
            })
            .transpose()?;
        let player_head_offset = player_head_dist
            .as_ref()
            .map(|dist| self.resources.rng.sample(dist) as Coord);
        let player_head = map.rect().top_left + player_head_offset;
        let player_facing_index =
            self.resources.rng.gen_range(0 .. Direction::ALL.len());
        let player_facing = Direction::ALL[player_facing_index];
        let player = PlayerPosition::new(player_head, player_facing)?;
        let game = Game::new(map, player)?;
        Ok(GeneratorState::Done(Some(game)))
    }
}

impl TaskProgress for Generator {
    fn progress_goal(&self) -> ProgressMetric {
        self.resources.map_gen.progress_goal() + 1
    }

    fn current_progress(&self) -> ProgressMetric {
        match &self.state {
            GeneratorState::GeneratingMap => {
                self.resources.map_gen.current_progress()
            },
            GeneratorState::GeneratingPlayer(_) => {
                self.resources.map_gen.progress_goal()
            },
            GeneratorState::Done(_) => self.progress_goal(),
        }
    }

    fn progress_status(&self) -> String {
        match &self.state {
            GeneratorState::GeneratingMap => {
                format!(
                    "generating map > {}",
                    self.resources.map_gen.progress_status()
                )
            },
            GeneratorState::GeneratingPlayer(_) => {
                "generating player".to_owned()
            },
            GeneratorState::Done(_) => "done".to_owned(),
        }
    }
}

impl TaskReset<GeneratorResetArgs> for Generator {
    type Output = ();
    type Error = Infallible;

    fn reset(
        &mut self,
        args: GeneratorResetArgs,
    ) -> Result<Self::Output, Self::Error> {
        self.state = GeneratorState::INITIAL;
        self.resources.rng = create_reproducible_rng(args.seed);
        self.resources.map_gen.reset(args.config.map_config)?;
        Ok(())
    }
}

impl<'a> TaskTick<&'a mut ()> for Generator {
    type Output = Game;
    type Error = GenError;

    fn on_tick(
        &mut self,
        tick: &mut Tick,
        args: &mut (),
    ) -> Result<Option<Self::Output>, Self::Error> {
        let current_state =
            mem::replace(&mut self.state, GeneratorState::INITIAL);
        self.state = self.transition(tick, args, current_state)?;
        match &mut self.state {
            GeneratorState::Done(output) => Ok(output.take()),
            _ => Ok(None),
        }
    }
}
