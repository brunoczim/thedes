use camera::Camera;
use rand::{SeedableRng, distr::Distribution, rngs::StdRng};
use thedes_domain::{
    event,
    game::{Game, MovePlayerError},
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
pub enum EventError {
    #[error("Failed to create event distribution")]
    Distr(
        #[from]
        #[source]
        gen_event::DistrError,
    ),
    #[error("Failed to apply event to game")]
    Apply(
        #[from]
        #[source]
        event::ApplyError,
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
}

#[derive(Debug, Error)]
pub enum QuickStepError {
    #[error("Failed to move player head")]
    MovePlayer(
        #[from]
        #[source]
        MovePlayerError,
    ),
}

#[derive(Debug, Clone)]
pub struct Config {
    camera: camera::Config,
    event_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self { camera: camera::Config::new(), event_interval: 3 }
    }

    pub fn with_camera(self, config: camera::Config) -> Self {
        Self { camera: config, ..self }
    }

    pub fn with_event_interval(self, ticks: u64) -> Self {
        Self { event_interval: ticks, ..self }
    }

    pub fn finish(self, game: Game) -> Session {
        Session {
            rng: StdRng::from_os_rng(),
            game,
            camera: self.camera.finish(),
            event_interval: self.event_interval,
            event_ticks: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    rng: StdRng,
    game: Game,
    camera: Camera,
    event_interval: u64,
    event_ticks: u64,
}

impl Session {
    pub fn render(&mut self, app: &mut App) -> Result<(), RenderError> {
        self.camera.render(app, &mut self.game)?;
        self.camera.update(app, &mut self.game);
        Ok(())
    }

    pub fn tick_event(&mut self) -> Result<(), EventError> {
        self.event_ticks += 1;
        if self.event_ticks >= self.event_interval {
            let event = EventDistr::new(&self.game)?.sample(&mut self.rng);
            event.apply(&mut self.game)?;
            self.event_interval = 0;
        }
        Ok(())
    }

    pub fn move_around(
        &mut self,
        app: &mut App,
        direction: Direction,
    ) -> Result<(), MoveAroundError> {
        self.game.move_player_pointer(direction)?;
        self.camera.update(app, &mut self.game);
        Ok(())
    }

    pub fn quick_step(
        &mut self,
        app: &mut App,
        direction: Direction,
    ) -> Result<(), QuickStepError> {
        self.game.move_player_head(direction)?;
        self.camera.update(app, &mut self.game);
        Ok(())
    }
}
