use std::{convert::Infallible, fmt};

use thedes_tui::{
    component::{
        menu::{self, Menu},
        task::{self, TaskMonitor},
        Cancellable,
        CancellableOutput,
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
    #[error("Error initializing session")]
    SessionInit(
        #[source]
        #[from]
        session::InitError,
    ),
    #[error("Error running session tick")]
    SessionTick(
        #[source]
        #[from]
        session::TickError,
    ),
    #[error("Failed to generate game")]
    GameGen(
        #[source]
        #[from]
        task::TickError<thedes_gen::game::GenError>,
    ),
    #[error("Failed to reset game state")]
    GameReset(
        #[source]
        #[from]
        task::ResetError<Infallible>,
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

#[derive(Debug, Clone)]
enum State {
    MainMenu,
    PlayMenu,
    Generating,
    Session(session::Component),
}

#[derive(Debug, Clone)]
pub struct Component {
    main_menu: Menu<MenuOption, NonCancellable>,
    state: State,
    gen_task: TaskMonitor<thedes_gen::game::Generator, Cancellable>,
    play_component: play::Component,
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
            gen_task: TaskMonitor::new(task::Config {
                base: task::BaseConfig::new("Generating game..."),
                cancellability: Cancellable::new(),
                task: thedes_gen::game::Config::new().finish(),
            }),
            state: State::MainMenu,
        })
    }

    pub fn on_tick(&mut self, tick: &mut Tick) -> Result<bool, TickError> {
        match &mut self.state {
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
                        play::Action::CreateGame(game) => {
                            self.state = State::Generating;
                            self.gen_task.reset(
                                thedes_gen::game::GeneratorResetArgs {
                                    seed: game.seed,
                                    config: thedes_gen::game::Config::new(),
                                },
                            )?;
                        },

                        play::Action::Cancel => {
                            self.state = State::MainMenu;
                        },
                    }
                    self.play_component.reset()?;
                }
            },

            State::Generating => {
                if let Some(output) = self.gen_task.on_tick(tick, &mut ())? {
                    self.state = match output {
                        CancellableOutput::Accepted(game) => {
                            State::Session(session::Component::new(game)?)
                        },
                        CancellableOutput::Cancelled => State::MainMenu,
                    };
                }
            },

            State::Session(session_component) => {
                if !session_component.on_tick(tick)? {
                    self.state = State::MainMenu;
                }
            },
        }

        Ok(true)
    }
}
