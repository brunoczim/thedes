use std::fmt;

use thedes_tui::{
    core::event::Key,
    menu::{self, Menu},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to initialize new game component")]
    NewGame(
        #[from]
        #[source]
        new_game::InitError,
    ),
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("Failed to run main menu")]
    MainMenu(
        #[from]
        #[source]
        menu::Error,
    ),
    #[error("Failed to run new game component")]
    NewGame(
        #[from]
        #[source]
        new_game::RunError,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuItem {
    Continue,
    Quit,
}

impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Continue => "Continue Game",
            Self::Quit => "Quit Game",
        })
    }
}

#[derive(Debug, Clone)]
pub struct Component {
    menu: Menu<MenuItem>,
}

impl Component {
    pub fn new() -> Result<Self, InitError> {
        let menu_items = [MenuItem::Continue, MenuItem::Quit];

        let quit_position = menu_items
            .iter()
            .position(|item| *item == MenuItem::Quit)
            .unwrap_or_default();

        let menu_bindings = menu::default_key_bindings()
            .with(Key::Char('q'), menu::Command::SelectConfirm(quit_position));

        let menu = Menu::new("=== T H E D E S ===", menu_items)?
            .with_keybindings(menu_bindings);

        Ok(Self { menu })
    }

    pub async fn run(
        &mut self,
        app: &mut thedes_tui::core::App,
    ) -> Result<(), RunError> {
        loop {
            self.menu.run(app).await?;

            match self.menu.output() {
                MainMenuItem::NewGame => {
                    self.new_game.run(app).await?;
                },
                MainMenuItem::LoadGame => {},
                MainMenuItem::Settings => {},
                MainMenuItem::Quit => break,
            }
        }

        Ok(())
    }
}
