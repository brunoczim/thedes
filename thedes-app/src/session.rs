use thedes_domain::game::Game;
use thedes_tui::Tick;
use thiserror::Error;

pub mod running;
pub mod paused;
mod command;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to initialize running component of game session")]
    Running(
        #[from]
        #[source]
        running::InitError,
    ),
}

#[derive(Debug, Error)]
pub enum TickError {
    #[error(transparent)]
    Paused(#[from] paused::TickError),
    #[error(transparent)]
    Running(#[from] running::TickError),
    #[error("Error resetting pause menu")]
    ResetPaused(
        #[from]
        #[source]
        paused::ResetError,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum State {
    Running,
    Paused,
}

#[derive(Debug, Clone)]
pub struct Component {
    state: State,
    running_component: running::Component,
    paused_component: paused::Component,
}

impl Component {
    pub fn new(game: Game) -> Result<Self, InitError> {
        Ok(Self {
            state: State::Running,
            running_component: running::Component::new(game)?,
            paused_component: paused::Component::new(),
        })
    }

    pub fn reset(&mut self) {
        self.state = State::Running;
    }

    pub fn on_tick(&mut self, tick: &mut Tick) -> Result<bool, TickError> {
        match self.state {
            State::Paused => match self.paused_component.on_tick(tick)? {
                Some(paused::Action::Resume) => {
                    self.state = State::Running;
                    Ok(true)
                },
                Some(paused::Action::Quit) => Ok(false),
                None => Ok(true),
            },
            State::Running => match self.running_component.on_tick(tick)? {
                Some(running::Action::Pause) => {
                    self.state = State::Paused;
                    self.paused_component.reset()?;
                    Ok(true)
                },
                None => Ok(true),
            },
        }
    }
}
