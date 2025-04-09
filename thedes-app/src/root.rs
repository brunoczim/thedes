use std::fmt;

use thedes_tui::{
    core::event::Key,
    menu::{self, Menu},
};
use thiserror::Error;

pub mod new_game;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to initialize main menu")]
    MainMenu(
        #[from]
        #[source]
        menu::Error,
    ),
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
enum MainMenuItem {
    NewGame,
    LoadGame,
    Settings,
    Quit,
}

impl fmt::Display for MainMenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NewGame => "New Game",
            Self::LoadGame => "Load Game",
            Self::Settings => "Settings",
            Self::Quit => "Quit",
        })
    }
}

#[derive(Debug, Clone)]
pub struct Component {
    main_menu: Menu<MainMenuItem>,
    new_game: new_game::Component,
}

impl Component {
    pub fn new() -> Result<Self, InitError> {
        let main_menu_items = [
            MainMenuItem::NewGame,
            MainMenuItem::LoadGame,
            MainMenuItem::Settings,
            MainMenuItem::Quit,
        ];

        let quit_position = main_menu_items
            .iter()
            .position(|item| *item == MainMenuItem::Quit)
            .unwrap_or_default();

        let main_menu_bindings = menu::default_key_bindings()
            .with(Key::Char('q'), menu::Command::SelectConfirm(quit_position));

        let main_menu = Menu::new("=== T H E D E S ===", main_menu_items)?
            .with_keybindings(main_menu_bindings);

        let new_game = new_game::Component::new()?;

        Ok(Self { main_menu, new_game })
    }

    pub async fn run(
        &mut self,
        app: &mut thedes_tui::core::App,
    ) -> Result<(), RunError> {
        loop {
            self.main_menu.run(app).await?;

            match self.main_menu.output() {
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
