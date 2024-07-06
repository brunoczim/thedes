use num::rational::Ratio;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum EventAction {
    Propagate(Action),
    Control(ControlAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ControlAction {
    MovePlayerHead(Direction),
    MovePlayerPointer(Direction),
}

fn arrow_key_to_direction(key: Key) -> Option<Direction> {
    Some(match key {
        Key::Up => Direction::Up,
        Key::Left => Direction::Left,
        Key::Down => Direction::Down,
        Key::Right => Direction::Right,
        _ => return None,
    })
}

#[derive(Debug, Clone)]
pub struct Component {
    first_render: bool,
    control_events_per_tick: Ratio<u64>,
    controls_left: Ratio<u64>,
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

        let control_events_per_tick = Ratio::new(1, 8);

        Ok(Self {
            first_render: true,
            control_events_per_tick,
            controls_left: control_events_per_tick,
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
        let more_controls_left =
            self.controls_left + self.control_events_per_tick;
        if more_controls_left < self.control_events_per_tick.ceil() * 2 {
            self.controls_left = more_controls_left;
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
            if let Some(event_action) = self.handle_input_event(event)? {
                match event_action {
                    EventAction::Propagate(action) => return Ok(Some(action)),
                    EventAction::Control(action) => {
                        if self.controls_left >= Ratio::ONE {
                            self.controls_left -= Ratio::ONE;
                            self.handle_control(action);
                        }
                    },
                }
            }
        }
        Ok(None)
    }

    fn handle_input_event(
        &mut self,
        event: Event,
    ) -> Result<Option<EventAction>, TickError> {
        match event {
            Event::Key(
                KeyEvent {
                    main_key: Key::Char('p') | Key::Char('P'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                }
                | KeyEvent { main_key: Key::Esc, .. },
            ) => Ok(Some(EventAction::Propagate(Action::Pause))),

            Event::Key(KeyEvent {
                main_key,
                ctrl,
                alt: false,
                shift: false,
            }) => {
                if let Some(direction) = arrow_key_to_direction(main_key) {
                    let action = if ctrl {
                        ControlAction::MovePlayerHead(direction)
                    } else {
                        ControlAction::MovePlayerPointer(direction)
                    };
                    Ok(Some(EventAction::Control(action)))
                } else {
                    Ok(None)
                }
            },

            _ => Ok(None),
        }
    }

    fn handle_control(&mut self, action: ControlAction) {
        match action {
            ControlAction::MovePlayerHead(direction) => {
                self.game.move_player_head(direction)
            },
            ControlAction::MovePlayerPointer(direction) => {
                self.game.move_player_pointer(direction)
            },
        }
    }
}
