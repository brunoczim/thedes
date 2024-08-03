use std::fmt;

use rand::{thread_rng, Rng};
use thedes_gen::random::Seed;
use thedes_tui::{
    component::{
        info::{self, InfoDialog},
        input::{self, InputDialog},
        menu::{self, Menu},
        Cancellability,
        Cancellable,
        CancellableOutput,
    },
    Tick,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Inconsistency setting maximum seed size")]
    MaxSeedSize(#[source] input::CursorOutOfBounds),
    #[error("Inconsistency setting maximum save name size")]
    MaxSaveNameSize(#[source] input::CursorOutOfBounds),
}

#[derive(Debug, Error)]
pub enum TickError {
    #[error(transparent)]
    RenderError(#[from] thedes_tui::CanvasError),
    #[error("Inconsistent main menu option list")]
    UnknownMenuOption(
        #[source]
        #[from]
        menu::UnknownOption<MenuOption>,
    ),
    #[error("Inconsistent save name buffer size")]
    SetSaveNameBuffer(#[source] input::InvalidNewBuffer),
    #[error("Inconsistent seed buffer size")]
    SetSeedBuffer(#[source] input::InvalidNewBuffer),
}

#[derive(Debug, Error)]
pub enum ResetError {
    #[error("Inconsistent save name buffer reset")]
    ResetSaveNameBuffer(#[source] input::InvalidNewBuffer),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameParams {
    pub name: String,
    pub seed: Seed,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    CreateGame(GameParams),
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MenuOption {
    Create,
    SetName,
    SetSeed,
}

impl fmt::Display for MenuOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::SetSeed => write!(f, "set seed"),
            Self::SetName => write!(f, "set save name"),
        }
    }
}

impl menu::OptionItem for MenuOption {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum State {
    Intro,
    Main,
    SetName,
    SetSeed,
    InvalidName { prev_valid: bool },
    InvalidSeed,
}

#[derive(Debug, Clone)]
pub struct Component {
    name: String,
    seed: Seed,
    menu: Menu<MenuOption, Cancellable>,
    save_name_input: InputDialog<fn(char) -> bool, Cancellable>,
    seed_input: InputDialog<fn(char) -> bool, Cancellable>,
    invalid_seed_info: InfoDialog,
    invalid_save_name_info: InfoDialog,
    state: State,
}

impl Component {
    pub fn new() -> Result<Self, InitError> {
        Ok(Self {
            name: String::new(),
            seed: 0,
            menu: Menu::new(menu::Config {
                base: menu::BaseConfig::new("NEW GAME"),
                cancellability: Cancellable::new(),
                options: menu::Options::with_initial(MenuOption::Create)
                    .add(MenuOption::SetName)
                    .add(MenuOption::SetSeed),
            }),
            save_name_input: InputDialog::new(input::Config {
                base: input::BaseConfig::new("NEW GAME NAME")
                    .with_max_graphemes(32)
                    .map_err(InitError::MaxSaveNameSize)?,
                cancellability: Cancellable::new(),
            }),
            seed_input: InputDialog::new(input::Config {
                base: input::BaseConfig::new("NEW GAME SEED")
                    .with_max_graphemes(8)
                    .map_err(InitError::MaxSeedSize)?,
                cancellability: Cancellable::new(),
            }),
            invalid_seed_info: InfoDialog::new(
                info::Config::new("Invalid seed!").with_message(
                    "Seeds must be composed of hexadecimal digits (0-9, a-f, \
                     A-F), having at least one digit",
                ),
            ),
            invalid_save_name_info: InfoDialog::new(
                info::Config::new("Invalid save name!")
                    .with_message("Save names cannot be empty"),
            ),
            state: State::Intro,
        })
    }

    pub fn reset(&mut self) -> Result<(), ResetError> {
        self.name.clear();
        self.state = State::Intro;
        self.save_name_input
            .set_buffer([])
            .map_err(ResetError::ResetSaveNameBuffer)?;
        Ok(())
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
    ) -> Result<Option<Action>, TickError> {
        match self.state {
            State::Intro => {
                if !self.save_name_input.on_tick(tick)? {
                    match self.save_name_input.selection() {
                        CancellableOutput::Accepted(name) => {
                            if name.is_empty() {
                                self.state =
                                    State::InvalidName { prev_valid: false };
                            } else {
                                self.seed = thread_rng().gen();
                                self.state = State::Main;
                                self.menu.select(MenuOption::Create)?;
                                self.seed_input
                                    .set_buffer(
                                        format!("{:x}", self.seed).chars(),
                                    )
                                    .map_err(TickError::SetSeedBuffer)?;
                            }
                        },

                        CancellableOutput::Cancelled => {
                            return Ok(Some(Action::Cancel))
                        },
                    }
                }
            },

            State::Main => {
                if !self.menu.on_tick(tick)? {
                    match self.menu.selection() {
                        CancellableOutput::Accepted(MenuOption::Create) => {
                            let action = Action::CreateGame(GameParams {
                                name: self.name.clone(),
                                seed: self.seed,
                            });
                            return Ok(Some(action));
                        },
                        CancellableOutput::Accepted(MenuOption::SetName) => {
                            self.state = State::SetName;
                        },
                        CancellableOutput::Accepted(MenuOption::SetSeed) => {
                            self.state = State::SetSeed;
                        },
                        CancellableOutput::Cancelled => {
                            return Ok(Some(Action::Cancel))
                        },
                    }
                }
            },

            State::SetName => {
                if !self.save_name_input.on_tick(tick)? {
                    if let CancellableOutput::Accepted(name) =
                        self.save_name_input.selection()
                    {
                        if name.is_empty() {
                            self.state =
                                State::InvalidName { prev_valid: true };
                        } else {
                            self.name = name;
                        }
                    } else {
                        self.save_name_input
                            .set_buffer(self.name.chars())
                            .map_err(TickError::SetSaveNameBuffer)?;
                        self.seed_input
                            .cancellability_mut()
                            .set_cancel_state(false);
                    }
                    self.state = State::Main;
                }
            },

            State::SetSeed => {
                if !self.seed_input.on_tick(tick)? {
                    self.state = State::Main;
                    if let CancellableOutput::Accepted(seed_str) =
                        self.seed_input.selection()
                    {
                        if let Ok(seed) = Seed::from_str_radix(&seed_str, 16) {
                            self.seed = seed;
                        } else {
                            self.state = State::InvalidSeed;
                        }
                    } else {
                        self.seed_input
                            .set_buffer(format!("{:x}", self.seed).chars())
                            .map_err(TickError::SetSeedBuffer)?;
                        self.seed_input
                            .cancellability_mut()
                            .set_cancel_state(false);
                    }
                }
            },

            State::InvalidName { prev_valid } => {
                if !self.invalid_seed_info.on_tick(tick)? {
                    self.state =
                        if prev_valid { State::Main } else { State::Intro };
                }
            },

            State::InvalidSeed => {
                if !self.invalid_save_name_info.on_tick(tick)? {
                    self.state = State::SetSeed;
                }
            },
        }

        Ok(None)
    }
}
