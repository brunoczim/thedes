use rand::Rng;
use rand_distr::{Triangular, TriangularError};
use thedes_async_util::progress;
use thedes_domain::{
    game::{self, Game},
    geometry::Coord,
    player::{self, PlayerPosition},
};
use thedes_geometry::orientation::{Axis, Direction};
use thiserror::Error;

use crate::{map, random::PickedReproducibleRng};

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Error initializing map generator")]
    Map(
        #[from]
        #[source]
        map::InitError,
    ),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error generating map")]
    Map(
        #[from]
        #[source]
        map::Error,
    ),
    #[error("Error creating random distribution for player's head in axis {1}")]
    PlayerHeadDistr(#[source] TriangularError, Axis),
    #[error("Failed to create a game")]
    Creation(
        #[source]
        #[from]
        game::InitError,
    ),
    #[error("Failed to create a player")]
    CreatePlayer(
        #[source]
        #[from]
        player::InitError,
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

    pub fn finish(
        self,
        rng: &mut PickedReproducibleRng,
    ) -> Result<Generator, InitError> {
        Ok(Generator { map_gen: self.map_config.finish(rng)? })
    }
}

#[derive(Debug)]
pub struct Generator {
    map_gen: map::Generator,
}

impl Generator {
    pub fn progress_goal(&self) -> usize {
        self.map_gen.progress_goal() + 1
    }

    pub async fn execute(
        self,
        rng: &mut PickedReproducibleRng,
        progress_logger: progress::Logger,
    ) -> Result<Game, Error> {
        progress_logger.set_status("generating map");
        let map = self.map_gen.execute(rng, progress_logger.nest()).await?;

        progress_logger.set_status("generating player");
        let player_head_distr = map
            .rect()
            .size
            .map_with_axes(|coord, axis| {
                let min = 2.0;
                let max = f64::from(coord) - 2.0 - f64::EPSILON;
                let mode = min + (max - min) / 2.0;
                Triangular::new(min, max, mode)
                    .map_err(|error| Error::PlayerHeadDistr(error, axis))
            })
            .transpose()?;
        let player_head_offset =
            player_head_distr.as_ref().map(|distr| rng.sample(distr) as Coord);
        let player_head = map.rect().top_left + player_head_offset;
        let player_facing_index = rng.random_range(0 .. Direction::ALL.len());
        let player_facing = Direction::ALL[player_facing_index];
        let player_pos = PlayerPosition::new(player_head, player_facing)?;
        let game = Game::new(map, player_pos)?;

        progress_logger.set_status("done");
        Ok(game)
    }
}
