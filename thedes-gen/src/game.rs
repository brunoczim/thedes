use rand::Rng;
use rand_distr::{Triangular, TriangularError};
use thedes_domain::{
    game::{self, Game},
    geometry::Coord,
    player::{self, Player},
};
use thedes_geometry::axis::{Axis, Direction};
use thiserror::Error;

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

    pub fn gen(
        self,
        rng: &mut PickedReproducibleRng,
    ) -> Result<Game, GenError> {
        let map = self.map_config.generate(rng)?;
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
        let player_head_offset =
            player_head_dist.as_ref().map(|dist| rng.sample(dist) as Coord);
        let player_head = map.rect().top_left + player_head_offset;
        let player_facing_index = rng.gen_range(0 .. Direction::ALL.len());
        let player_facing = Direction::ALL[player_facing_index];
        let player = Player::new(player_head, player_facing)?;
        let game = Game::new(map, player)?;
        Ok(game)
    }
}

#[derive(Debug)]
pub struct Generator {
    config: Config,
}
