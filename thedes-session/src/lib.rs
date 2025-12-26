use camera::Camera;
use num::rational::Ratio;
use rand::{SeedableRng, distr::Distribution, rngs::StdRng};
use thedes_domain::{
    event,
    game2::{self, Game2, MovePlayerError},
};
use thedes_gen::event::{self as gen_event, EventDistr};
use thedes_geometry::orientation::Direction;
use thedes_tui::core::App;

use thiserror::Error;

pub mod camera;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to handle session camera")]
    Camera(
        #[from]
        #[source]
        camera::Error,
    ),
}

#[derive(Debug, Error)]
pub enum TickError {
    #[error("Failed to tick game")]
    Game(
        #[from]
        #[source]
        game2::TickError,
    ),
}

#[derive(Debug, Error)]
pub enum MoveAroundError {
    #[error("Failed to move player pointer")]
    MovePlayer(
        #[from]
        #[source]
        MovePlayerError,
    ),
    #[error("Failed to update camera")]
    QuickStep(
        #[from]
        #[source]
        camera::Error,
    ),
}

#[derive(Debug, Error)]
pub enum QuickStepError {
    #[error("Failed to move player head")]
    MovePlayer(
        #[from]
        #[source]
        MovePlayerError,
    ),
    #[error("Failed to update camera")]
    QuickStep(
        #[from]
        #[source]
        camera::Error,
    ),
}

#[derive(Debug, Clone)]
pub struct Config {
    camera: camera::Config,
    event_interval: Ratio<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            camera: camera::Config::new(),
            event_interval: Ratio::new(1, 100),
        }
    }

    pub fn with_camera(self, config: camera::Config) -> Self {
        Self { camera: config, ..self }
    }

    pub fn with_event_interval(self, ticks: Ratio<u64>) -> Self {
        Self { event_interval: ticks, ..self }
    }

    pub fn finish(self, game: Game2) -> Session {
        Session {
            rng: StdRng::from_os_rng(),
            game,
            camera: self.camera.finish(),
            event_interval: self.event_interval,
            event_ticks: Ratio::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    rng: StdRng,
    game: Game2,
    camera: Camera,
    event_interval: Ratio<u64>,
    event_ticks: Ratio<u64>,
}

impl Session {
    pub fn render(&mut self, app: &mut App) -> Result<(), RenderError> {
        self.camera.render(app, &mut self.game)?;
        self.camera.update(app, &mut self.game)?;
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), TickError> {
        self.game.tick()?;
        Ok(())
    }

    pub fn move_around(
        &mut self,
        app: &mut App,
        direction: Direction,
    ) -> Result<(), MoveAroundError> {
        self.game.move_player(direction)?;
        self.camera.update(app, &mut self.game)?;
        Ok(())
    }

    pub fn quick_step(
        &mut self,
        app: &mut App,
        direction: Direction,
    ) -> Result<(), QuickStepError> {
        self.game.move_player(direction)?;
        self.camera.update(app, &mut self.game)?;
        Ok(())
    }
}
