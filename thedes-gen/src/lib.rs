use random::{PickedReproducibleRng, Seed, create_reproducible_rng};
use thedes_async_util::progress;
use thedes_domain::game::Game;
use thiserror::Error;

pub mod random;
pub mod matter;
pub mod map;
pub mod game;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to initialize game generator")]
    Game(
        #[source]
        #[from]
        game::InitError,
    ),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to generate game")]
    Game(
        #[source]
        #[from]
        game::Error,
    ),
}

#[derive(Debug)]
pub struct Config {
    game: game::Config,
    seed: Seed,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self { game: game::Config::new(), seed: 1 }
    }

    pub fn with_game(self, config: game::Config) -> Self {
        Self { game: config, ..self }
    }

    pub fn with_seed(self, seed: Seed) -> Self {
        Self { seed, ..self }
    }

    pub fn finish(self) -> Result<Generator, InitError> {
        let mut rng = create_reproducible_rng(self.seed);
        let game_gen = self.game.finish(&mut rng)?;
        let goal = game_gen.progress_goal();
        let (progress_logger, progress_monitor) = progress::open(goal);
        Ok(Generator { game_gen, rng, progress_monitor, progress_logger })
    }
}

#[derive(Debug)]
pub struct Generator {
    game_gen: game::Generator,
    rng: PickedReproducibleRng,
    progress_monitor: progress::Monitor,
    progress_logger: progress::Logger,
}

impl Generator {
    pub fn progress_monitor(&self) -> progress::Monitor {
        self.progress_monitor.clone()
    }

    pub async fn execute(mut self) -> Result<Game, Error> {
        let game =
            self.game_gen.execute(&mut self.rng, self.progress_logger).await?;
        Ok(game)
    }
}
