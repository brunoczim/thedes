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
    #[error(transparent)]
    Command(#[from] command::TickError),
    #[error("Error resetting pause menu")]
    ResetPaused(
        #[from]
        #[source]
        paused::ResetError,
    ),
    #[error("Error resetting debug commands menu")]
    ResetCommands(
        #[from]
        #[source]
        command::ResetError,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum State {
    Running,
    Paused,
    ChoosingCommands,
}

#[derive(Debug, Clone)]
pub struct Component {
    game: Game,
    state: State,
    running_component: running::Component,
    paused_component: paused::Component,
    command_component: command::Component,
}

impl Component {
    pub fn new(game: Game) -> Result<Self, InitError> {
        Ok(Self {
            game,
            state: State::Running,
            running_component: running::Component::new()?,
            paused_component: paused::Component::new(),
            command_component: command::Component::new(),
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
            State::Running => {
                match self.running_component.on_tick(tick, &mut self.game)? {
                    Some(running::Action::Pause) => {
                        self.state = State::Paused;
                        self.paused_component.reset()?;
                        Ok(true)
                    },
                    Some(running::Action::ChooseCommands) => {
                        self.state = State::ChoosingCommands;
                        self.command_component.reset()?;
                        Ok(true)
                    },
                    None => Ok(true),
                }
            },
            State::ChoosingCommands => {
                if !self.command_component.on_tick(tick, &mut self.game)? {
                    self.state = State::Running;
                }
                Ok(true)
            },
        }
    }
}
