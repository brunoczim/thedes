use std::fmt;

use thedes_tui::{
    component::{
        menu::{self, Menu},
        Cancellable,
    },
    Tick,
};
use thiserror::Error;

pub mod new;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to initialize new game components")]
    NewGame(
        #[from]
        #[source]
        new::InitError,
    ),
}

#[derive(Debug, Error)]
pub enum TickError {
    #[error(transparent)]
    RenderError(#[from] thedes_tui::RenderError),
    #[error("New game component tick failed")]
    New(
        #[from]
        #[source]
        new::TickError,
    ),
    #[error("Error reseting new game components")]
    ResetNew(
        #[source]
        #[from]
        new::ResetError,
    ),
}

#[derive(Debug, Error)]
pub enum ResetError {
    #[error("Error reseting new game components")]
    New(
        #[source]
        #[from]
        new::ResetError,
    ),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    CreateGame(new::Game),
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MenuOption {
    New,
    Load,
    Delete,
}

impl fmt::Display for MenuOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::New => write!(f, "new game"),
            Self::Load => write!(f, "load game"),
            Self::Delete => write!(f, "delete game"),
        }
    }
}

impl menu::OptionItem for MenuOption {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum State {
    Main,
    New,
    Load,
    Delete,
}

#[derive(Debug, Clone)]
pub struct Component {
    menu: Menu<MenuOption, Cancellable>,
    new_game_component: new::Component,
    state: State,
}

impl Component {
    pub fn new() -> Result<Self, InitError> {
        Ok(Self {
            menu: Menu::new(menu::Config {
                base: menu::BaseConfig::new("GAMES"),
                options: menu::Options::with_initial(MenuOption::New)
                    .add(MenuOption::Load)
                    .add(MenuOption::Delete),
                cancellability: Cancellable::new(),
            }),
            new_game_component: new::Component::new()?,
            state: State::Main,
        })
    }

    pub fn reset(&mut self) -> Result<(), ResetError> {
        self.state = State::Main;
        self.new_game_component.reset()?;
        Ok(())
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
    ) -> Result<Option<Action>, TickError> {
        match self.state {
            State::Main => {
                if !self.menu.on_tick(tick)? {
                    match self.menu.selection() {
                        Some(MenuOption::New) => {
                            self.state = State::New;
                        },
                        Some(MenuOption::Load) => {
                            self.state = State::Load;
                        },
                        Some(MenuOption::Delete) => {
                            self.state = State::Delete;
                        },
                        None => return Ok(Some(Action::Cancel)),
                    }
                }
            },

            State::New => {
                if let Some(action) = self.new_game_component.on_tick(tick)? {
                    match action {
                        new::Action::CreateGame(game) => {
                            return Ok(Some(Action::CreateGame(game)))
                        },
                        new::Action::Cancel => {
                            self.state = State::Main;
                        },
                    }
                }
            },

            State::Load => todo!(),

            State::Delete => todo!(),
        }

        Ok(None)
    }
}
