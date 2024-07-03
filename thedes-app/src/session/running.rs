use rand::SeedableRng;
use thedes_domain::{
    game::Game,
    gen::{self, GameGenError, PickedReproducibleRng},
};
use thedes_geometry::axis::Direction;
use thedes_graphics::camera::{self, Camera};
use thedes_tui::{
    event::{Event, Key, KeyEvent},
    Tick,
};
use thiserror::Error;

use crate::play::new::Seed;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to generate game")]
    Gen(
        #[from]
        #[source]
        GameGenError,
    ),
}

#[derive(Debug, Error)]
pub enum TickError {
    #[error(transparent)]
    RenderError(#[from] thedes_tui::CanvasError),
    #[error("Error happened while rendering game on-camera")]
    Camera(
        #[from]
        #[source]
        camera::Error,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    Pause,
}

#[derive(Debug, Clone)]
pub struct Component {
    first_render: bool,
    camera: Camera,
    game: Game,
}

impl Component {
    pub fn new(seed: Seed) -> Result<Self, InitError> {
        let mut full_seed =
            <PickedReproducibleRng as SeedableRng>::Seed::default();
        for (i, chunk) in full_seed.chunks_exact_mut(4).enumerate() {
            let i = i as Seed;
            let bits = seed.wrapping_sub(i) ^ (i << 14);
            chunk.copy_from_slice(&bits.to_le_bytes());
        }

        let mut reproducible_rng = PickedReproducibleRng::from_seed(full_seed);

        Ok(Self {
            first_render: true,
            camera: camera::Config::new().finish(),
            game: gen::GameConfig::new().gen(&mut reproducible_rng)?,
        })
    }

    pub fn reset(&mut self) {
        self.first_render = true;
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
    ) -> Result<Option<Action>, TickError> {
        if !self.first_render {
            if let Some(action) = self.handle_input(tick)? {
                return Ok(Some(action));
            }
        }
        self.camera.on_tick(tick, &self.game)?;
        self.first_render = false;
        Ok(None)
    }

    fn handle_input(
        &mut self,
        tick: &mut Tick,
    ) -> Result<Option<Action>, TickError> {
        while let Some(event) = tick.next_event() {
            if let Some(action) = self.handle_input_event(event)? {
                return Ok(Some(action));
            }
        }
        Ok(None)
    }

    fn handle_input_event(
        &mut self,
        event: Event,
    ) -> Result<Option<Action>, TickError> {
        match event {
            Event::Key(key) => match key {
                KeyEvent {
                    main_key: Key::Char('q') | Key::Char('Q'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                }
                | KeyEvent { main_key: Key::Esc, .. } => {
                    return Ok(Some(Action::Pause))
                },

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

        Ok(None)
    }
}
