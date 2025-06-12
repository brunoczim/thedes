use camera::Camera;
use thedes_domain::game::{Game, MovePlayerError};
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
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self { camera: camera::Config::new() }
    }

    pub fn with_camera(self, config: camera::Config) -> Self {
        Self { camera: config, ..self }
    }

    pub fn finish(self, game: Game) -> Session {
        Session { game, camera: self.camera.finish() }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    game: Game,
    camera: Camera,
}

impl Session {
    pub fn render(&mut self, app: &mut App) -> Result<(), RenderError> {
        self.camera.render(app, &mut self.game)?;
        self.camera.update(app, &mut self.game);
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
