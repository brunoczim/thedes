use thedes_domain::game::Game;
use thedes_tui::Tick;
use thiserror::Error;

pub mod running;
pub mod paused;
mod script;

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
    Command(#[from] script::TickError),
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
    ChoosingCommands,
}

#[derive(Debug, Clone)]
pub struct Component {
    game: Game,
    state: State,
    running_component: running::Component,
    paused_component: paused::Component,
    script_component: script::Component,
}

impl Component {
    pub fn new(game: Game) -> Result<Self, InitError> {
        Ok(Self {
            game,
            state: State::Running,
            running_component: running::Component::new()?,
            paused_component: paused::Component::new(),
            script_component: script::Component::new(),
        })
    }

    pub fn reset(&mut self) {
        self.state = State::Running;
        self.script_component.reset();
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
                    Some(running::Action::ChooseScript) => {
                        self.state = State::ChoosingCommands;
                        Ok(true)
                    },
                    Some(running::Action::RunPreviousScript) => {
                        self.script_component.run_previous(&mut self.game);
                        Ok(true)
                    },
                    None => Ok(true),
                }
            },
            State::ChoosingCommands => {
                if !self.script_component.on_tick(tick, &mut self.game)? {
                    self.state = State::Running;
                }
                Ok(true)
            },
        }
    }
}
