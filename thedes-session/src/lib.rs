use thedes_domain::game::{self, Game};
use thedes_geometry::axis::Direction;
use thedes_graphics::camera::{self, Camera, CameraError};
use thedes_tui::{
    event::{Event, Key, KeyEvent},
    Tick,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Error happened while rendering game on-camera")]
    Camera(
        #[from]
        #[source]
        CameraError,
    ),
}

#[derive(Debug, Clone)]
pub struct Config {
    camera_config: camera::Config,
    game_config: game::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            camera_config: camera::Config::new(),
            game_config: game::Config::new(),
        }
    }

    pub fn with_camera(self, camera_config: camera::Config) -> Self {
        Self { camera_config, ..self }
    }

    pub fn with_game(self, game_config: game::Config) -> Self {
        Self { game_config, ..self }
    }

    pub fn finish(self) -> Session {
        Session::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    first_render: bool,
    camera: Camera,
    game: Game,
}

impl Session {
    fn new(config: Config) -> Self {
        Self {
            first_render: true,
            camera: config.camera_config.finish(),
            game: config.game_config.finish(),
        }
    }

    pub fn reset(&mut self) {
        self.first_render = true;
    }

    pub fn on_tick(&mut self, tick: &mut Tick) -> Result<bool, SessionError> {
        if !self.first_render && !self.handle_input(tick)? {
            return Ok(false);
        }
        self.camera.on_tick(tick, &self.game)?;
        self.first_render = false;
        Ok(true)
    }

    pub fn handle_input(
        &mut self,
        tick: &mut Tick,
    ) -> Result<bool, SessionError> {
        while let Some(event) = tick.next_event() {
            if !self.handle_input_event(event)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn handle_input_event(
        &mut self,
        event: Event,
    ) -> Result<bool, SessionError> {
        match event {
            Event::Key(key) => match key {
                KeyEvent {
                    main_key: Key::Char('q') | Key::Char('Q'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                }
                | KeyEvent { main_key: Key::Esc, .. } => return Ok(false),

                KeyEvent {
                    main_key: Key::Up,
                    ctrl: false,
                    alt: false,
                    shift: false,
                } => self.game.move_player_pointer(Direction::Up),
                KeyEvent {
                    main_key: Key::Left,
                    ctrl: false,
                    alt: false,
                    shift: false,
                } => self.game.move_player_pointer(Direction::Left),
                KeyEvent {
                    main_key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                } => self.game.move_player_pointer(Direction::Down),
                KeyEvent {
                    main_key: Key::Right,
                    ctrl: false,
                    alt: false,
                    shift: false,
                } => self.game.move_player_pointer(Direction::Right),

                KeyEvent {
                    main_key: Key::Up,
                    ctrl: true,
                    alt: false,
                    shift: false,
                } => self.game.move_player_head(Direction::Up),
                KeyEvent {
                    main_key: Key::Left,
                    ctrl: true,
                    alt: false,
                    shift: false,
                } => self.game.move_player_head(Direction::Left),
                KeyEvent {
                    main_key: Key::Down,
                    ctrl: true,
                    alt: false,
                    shift: false,
                } => self.game.move_player_head(Direction::Down),
                KeyEvent {
                    main_key: Key::Right,
                    ctrl: true,
                    alt: false,
                    shift: false,
                } => self.game.move_player_head(Direction::Right),

                _ => (),
            },

            Event::Paste(_) => (),
        }

        Ok(true)
    }
}
