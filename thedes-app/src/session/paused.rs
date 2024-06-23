use std::fmt;

use thedes_tui::{
    component::{
        menu::{self, Menu},
        NonCancellable,
    },
    Tick,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TickError {
    #[error(transparent)]
    RenderError(#[from] thedes_tui::CanvasError),
}

#[derive(Debug, Error)]
#[error("Inconsistent lookup of menu option")]
pub struct ResetError {
    #[from]
    source: menu::UnknownOption<MenuOption>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    Resume,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MenuOption {
    Resume,
    Quit,
}

impl fmt::Display for MenuOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resume => write!(f, "resume game"),
            Self::Quit => write!(f, "quit game"),
        }
    }
}

impl menu::OptionItem for MenuOption {}

#[derive(Debug, Clone)]
pub struct Component {
    menu: Menu<MenuOption, NonCancellable>,
}

impl Component {
    pub fn new() -> Self {
        Self {
            menu: Menu::new(menu::Config {
                base: menu::BaseConfig::new("Game is paused."),
                options: menu::Options::with_initial(MenuOption::Resume)
                    .add(MenuOption::Quit),
                cancellability: NonCancellable,
            }),
        }
    }

    pub fn reset(&mut self) -> Result<(), ResetError> {
        self.menu.select(MenuOption::Resume)?;
        Ok(())
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
    ) -> Result<Option<Action>, TickError> {
        if self.menu.on_tick(tick)? {
            Ok(None)
        } else {
            match self.menu.selection() {
                MenuOption::Resume => Ok(Some(Action::Resume)),
                MenuOption::Quit => Ok(Some(Action::Quit)),
            }
        }
    }
}
