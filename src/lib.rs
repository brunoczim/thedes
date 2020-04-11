/// Exports macros.
#[macro_use]
pub mod macros;

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
    storage::save,
    ui::{InputDialog, Menu, MenuOption},
};

/// Game app's start point.
pub async fn game_main(term: terminal::Handle) -> Result<()> {
    let main_menu = MainMenuOption::menu();

    loop {
        match main_menu.select(&term).await? {
            MainMenuOption::NewGame => new_game(&term).await?,
            MainMenuOption::LoadGame => load_game(&term).await?,
            MainMenuOption::DeleteGame => delete_game(&term).await?,
            MainMenuOption::Exit => break,
        }
    }

    Ok(())
}

/// Handles when a new game is asked.
pub async fn new_game(term: &terminal::Handle) -> Result<()> {
    let mut dialog = InputDialog::new(
        graphemes!["== New Game  =="],
        graphemes![],
        save::MAX_NAME,
        save::is_valid_name_char,
    );
    let input = dialog.select_with_cancel(term).await?;
    if let Some(stem) = input {
        if stem.len() == 0 {
            let dialog = InfoDialog::new(
                graphemes!["A Save Name Cannot Be Empty"],
                graphemes![
                    "Your input was empty. It cannot be empty for a save name."
                ],
            );
            dialog.run(term).await?;
        } else {
            let name = save::SaveName::from_stem(&stem).await?;
            let game = name
                .new_game(Seed::random())
                .await
                .prefix(|| format!("Error creating game {}", stem))?;

            let mut session =
                Session::new(game, name.clone()).await.prefix(|| {
                    format!("Error running game {}", name.printable())
                })?;
            session.game_loop(term).await.prefix(|| {
                format!("Error running game {}", name.printable())
            })?;
        }
    }
}

/// Handles when a game is asked to be loaded.
pub async fn load_game(term: &terminal::Handle) -> Result<()> {
    Ok(())
}

/// Handles when a game is asked to be deleted.
pub async fn delete_game(term: &terminal::Handle) -> Result<()> {
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
            graphemes!["=== T H E D E S ==="],
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
        graphemes![string]
    }
}
