/// Exports error utilites.
pub mod error;

/// Exports graphics related utilites.
pub mod graphics;

/// Exports coordinates related items, such as [coord::Axis], [coord::Point],
/// etc.
pub mod coord;

/// Exports input events such as [input::Key] and [input::Resize].
pub mod input;

/// Exports terminal handle and terminal related items.
pub mod terminal;

/// (T)UI related utilities, such as menu, dialogs, etc.
pub mod ui;

/// Storage related functions, such as directories and saved games.
pub mod storage;

use crate::{
    error::Result,
    graphics::Grapheme,
    ui::{Menu, MenuOption},
};

/// Game app's start point.
pub async fn game_main(handle: terminal::Handle) -> Result<()> {
    let main_menu = MainMenuOption::menu();

    loop {
        match main_menu.select(&handle).await? {
            MainMenuOption::NewGame => {},
            MainMenuOption::LoadGame => {},
            MainMenuOption::DeleteGame => {},
            MainMenuOption::Exit => break,
        }
    }

    Ok(())
}

/// An option of the game's main menu.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MainMenuOption {
    NewGame,
    LoadGame,
    DeleteGame,
    Exit,
}

impl MainMenuOption {
    pub fn menu() -> Menu<Self> {
        Menu::new(
            Grapheme::expect_iter("=== T H E D E S ===").collect(),
            vec![
                MainMenuOption::NewGame,
                MainMenuOption::LoadGame,
                MainMenuOption::DeleteGame,
                MainMenuOption::Exit,
            ],
        )
    }
}

impl MenuOption for MainMenuOption {
    fn name(&self) -> Vec<Grapheme> {
        let string = match self {
            MainMenuOption::NewGame => "NEW GAME",
            MainMenuOption::LoadGame => "LOAD GAME",
            MainMenuOption::DeleteGame => "DELETE GAME",
            MainMenuOption::Exit => "EXIT",
        };
        Grapheme::expect_iter(string).collect()
    }
}
