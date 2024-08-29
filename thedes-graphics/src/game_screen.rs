use thedes_domain::game::Game;
use thedes_geometry::CoordPair;
use thedes_tui::{CanvasError, TextStyle, Tick};
use thiserror::Error;

use crate::camera::{self, Camera};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error manipulating camera")]
    Camera(
        #[from]
        #[source]
        camera::Error,
    ),
    #[error("Error rendering info")]
    Canvas(
        #[from]
        #[source]
        CanvasError,
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

    pub fn with_camera(self, camera_config: camera::Config) -> Self {
        Self { camera: camera_config, ..self }
    }

    pub fn finish(self) -> GameScreen {
        GameScreen::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct GameScreen {
    camera: Camera,
}

impl GameScreen {
    fn new(config: Config) -> Self {
        Self { camera: config.camera.finish() }
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
        game: &Game,
    ) -> Result<(), Error> {
        let camera_dynamic_style = camera::DynamicStyle {
            margin_top_left: CoordPair { y: 1, x: 0 },
            margin_bottom_right: CoordPair { y: 0, x: 0 },
        };
        self.camera.on_tick(tick, game, &camera_dynamic_style)?;

        let pos_string = format!("↱{}", game.player().head());
        tick.screen_mut().styled_text(&pos_string, &TextStyle::default())?;

        Ok(())
    }
}
