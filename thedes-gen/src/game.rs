use std::{convert::Infallible, mem};

use rand::Rng;
use rand_distr::{Triangular, TriangularError};
use thedes_domain::{
    game::{self, Game},
    geometry::Coord,
    map::Map,
    player::{self, Player},
};
use thedes_geometry::axis::{Axis, Direction};
use thedes_tui::component::task::{ProgressMetric, Task};
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
            state: GeneratorState::GeneratingMap,
            rng: create_reproducible_rng(0),
            map_gen: self.map_config.finish(),
        }
    }
}

#[derive(Debug, Clone)]
enum GeneratorState {
    GeneratingMap,
    GeneratingPlayer(Map),
    Done,
}

#[derive(Debug, Clone)]
pub struct Generator {
    state: GeneratorState,
    rng: PickedReproducibleRng,
    map_gen: map::Generator,
}

impl<'a> Task<'a> for Generator {
    type ResetArgs = (Seed, Config);
    type ResetOutput = ();
    type ResetError = Infallible;
    type TickArgs = ();
    type TickOutput = Game;
    type TickError = GenError;

    #[inline(always)]
    fn progress_goal(&self) -> ProgressMetric {
        self.map_gen.progress_goal() + 1
    }

    #[inline(always)]
    fn progress_status(&self) -> ProgressMetric {
        match &self.state {
            GeneratorState::GeneratingMap => self.map_gen.progress_status(),
            GeneratorState::GeneratingPlayer(_) => self.map_gen.progress_goal(),
            GeneratorState::Done => self.progress_goal(),
        }
    }

    fn reset(
        &mut self,
        (seed, config): Self::ResetArgs,
    ) -> Result<Self::ResetOutput, Self::ResetError> {
        self.state = GeneratorState::GeneratingMap;
        self.rng = create_reproducible_rng(seed);
        self.map_gen.reset(config.map_config)?;
        Ok(())
    }

    #[inline(always)]
    fn on_tick(
        &mut self,
        tick: &mut thedes_tui::Tick,
        _args: &mut Self::TickArgs,
    ) -> Result<Option<Self::TickOutput>, Self::TickError> {
        loop {
            match mem::replace(&mut self.state, GeneratorState::GeneratingMap) {
                GeneratorState::Done => {},
                GeneratorState::GeneratingMap => {
                    self.state =
                        match self.map_gen.on_tick(tick, &mut self.rng)? {
                            Some(map) => GeneratorState::GeneratingPlayer(map),
                            None => GeneratorState::GeneratingMap,
                        };
                    break Ok(None);
                },
                GeneratorState::GeneratingPlayer(map) => {
                    let player_head_dist = map
                        .rect()
                        .size
                        .map_with_axes(|coord, axis| {
                            let min = 2.0;
                            let max = f64::from(coord) - 2.0 - f64::EPSILON;
                            let mode = min + (max - min) / 2.0;
                            Triangular::new(min, max, mode).map_err(|error| {
                                GenError::PlayerHeadDist(error, axis)
                            })
                        })
                        .transpose()?;
                    let player_head_offset = player_head_dist
                        .as_ref()
                        .map(|dist| self.rng.sample(dist) as Coord);
                    let player_head = map.rect().top_left + player_head_offset;
                    let player_facing_index =
                        self.rng.gen_range(0 .. Direction::ALL.len());
                    let player_facing = Direction::ALL[player_facing_index];
                    let player = Player::new(player_head, player_facing)?;
                    let game = Game::new(map, player)?;
                    self.state = GeneratorState::Done;
                    break Ok(Some(game));
                },
            }
        }
    }
}
