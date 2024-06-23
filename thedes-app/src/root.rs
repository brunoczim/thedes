use std::fmt;

use thedes_tui::{
    component::{
        menu::{self, Menu},
        NonCancellable,
    },
    Tick,
};
use thiserror::Error;

use crate::{play, session};

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to initialize play components")]
    Play(
        #[from]
        #[source]
        play::InitError,
    ),
}

#[derive(Debug, Error)]
pub enum TickError {
    #[error(transparent)]
    RenderError(#[from] thedes_tui::CanvasError),
    #[error("Play game component tick failed")]
    Play(
        #[from]
        #[source]
        play::TickError,
    ),
    #[error("Error reseting play game components")]
    ResetPlay(
        #[source]
        #[from]
        play::ResetError,
    ),
    #[error("Error running session tick")]
    Session(
        #[source]
        #[from]
        session::TickError,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MenuOption {
    Play,
    Settings,
    Help,
    Quit,
}

impl fmt::Display for MenuOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Play => write!(f, "play"),
            Self::Settings => write!(f, "settings"),
            Self::Help => write!(f, "help"),
            Self::Quit => write!(f, "quit"),
        }
    }
}

impl menu::OptionItem for MenuOption {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum State {
    MainMenu,
    PlayMenu,
    Session,
}

#[derive(Debug, Clone)]
pub struct Component {
    main_menu: Menu<MenuOption, NonCancellable>,
    state: State,
    play_component: play::Component,
    session_component: session::Component,
}

impl Component {
    pub fn new() -> Result<Self, InitError> {
        Ok(Self {
            main_menu: Menu::new(menu::Config {
                base: menu::BaseConfig::new("=== T H E D E S ==="),
                cancellability: NonCancellable,
                options: menu::Options::with_initial(MenuOption::Play)
                    .add(MenuOption::Settings)
                    .add(MenuOption::Help)
                    .add(MenuOption::Quit),
            }),
            play_component: play::Component::new()?,
            state: State::MainMenu,
            session_component: session::Component::new(),
        })
    }

    pub fn on_tick(&mut self, tick: &mut Tick) -> Result<bool, TickError> {
        match self.state {
            State::MainMenu => {
                if !self.main_menu.on_tick(tick)? {
                    match self.main_menu.selection() {
                        MenuOption::Play => {
                            self.state = State::PlayMenu;
                        },
                        MenuOption::Help => todo!(),
                        MenuOption::Settings => todo!(),
                        MenuOption::Quit => return Ok(false),
                    }
                }
            },

            State::PlayMenu => {
                if let Some(action) = self.play_component.on_tick(tick)? {
                    match action {
                        play::Action::CreateGame(_game) => {
                            self.state = State::Session;
                            self.session_component.reset();
                        },

                        play::Action::Cancel => {
                            self.state = State::MainMenu;
                        },
                    }
                    self.play_component.reset()?;
                }
            },

            State::Session => {
                if !self.session_component.on_tick(tick)? {
                    self.state = State::MainMenu;
                }
            },
        }

        Ok(true)
    }
}
