#![recursion_limit = "256"]
#![deny(unused_must_use)]

/// Asynchronous detaching utilities.
pub mod detach;

/// Iterator extensions.
pub mod iter_ext;

/// Random number generation utilites.
pub mod rand;

/// Error handling.
pub mod error;

/// Contains items related to terminal handling.
pub mod terminal;

/// Contains items related to orientation in the game.
pub mod orient;

/// Contains items related to rendering on the screen.
pub mod render;

/// Contains items related to user input.
pub mod input;

/// Storage related functions, such as directories and saved games.
pub mod storage;

/// Contains utilites for handling uis.
pub mod ui;

/*
/// Contains items related to current player handling.
pub mod player;

/// Terminal handling utilites.
pub mod term;

/// Contains items related to the map of the game.
pub mod map;

/// Contains data related to game sessions (ongoing games).
pub mod session;
*/

use crate::{
    error::{GameResult, ResultExt},
    render::TextSettings,
    storage::save,
    ui::{
        menu_select,
        menu_select_with_cancel,
        DangerPrompt,
        DangerPromptItem,
        InfoDialog,
        InputDialog,
        MainMenu,
        MainMenuItem,
    },
};

/// The 'top' function for the game.
pub async fn game_main() -> GameResult<()> {
    let mut term = terminal::Handle::new().await?;

    loop {
        match menu_select(&MainMenu, &mut term).await? {
            MainMenuItem::NewGame => {
                if let Err(err) = new_game(&mut term).await {
                    let dialog = InfoDialog {
                        title: "Error Creating New Game",
                        message: &format!("{}", err),
                        settings: TextSettings::new().align(1, 2),
                    };
                    dialog.run(&mut term).await?;
                }
            },

            MainMenuItem::LoadGame => {
                if let Err(err) = load_game(&mut term).await {
                    let dialog = InfoDialog {
                        title: "Error Loading Game",
                        message: &format!("{}", err),
                        settings: TextSettings::new().align(1, 2),
                    };
                    dialog.run(&mut term).await?;
                }
            },

            MainMenuItem::DeleteGame => {
                if let Err(err) = delete_game(&mut term).await {
                    let dialog = InfoDialog {
                        title: "Error Deleting New Game",
                        message: &format!("{}", err),
                        settings: TextSettings::new().align(1, 2),
                    };
                    dialog.run(&mut term).await?;
                }
            },

            MainMenuItem::Exit => break Ok(()),
        }
    }
}

/// Handles when a new game is asked.
pub async fn new_game(term: &mut terminal::Handle) -> GameResult<()> {
    let mut dialog = InputDialog::new(
        "== New Game  ==",
        "",
        save::MAX_NAME,
        save::is_valid_name_char,
    );
    let input = dialog.select_with_cancel(term).await?;
    if let Some(stem) = input {
        let name = save::SaveName::from_stem(&stem).await?;
        let game = name
            .new_game()
            .await
            .prefix(|| format!("Error creating game {}", stem))?;
    }

    Ok(())
}

/// Handles when a game is asked to be loaded.
pub async fn load_game(term: &mut terminal::Handle) -> GameResult<()> {
    let saves = save::list().await?;
    if saves.len() == 0 {
        let dialog = InfoDialog {
            title: "No saved games found",
            message: &format!(
                "No saved games were found at {}",
                save::path()?.display()
            ),
            settings: TextSettings::new().align(1, 2),
        };
        dialog.run(term).await?;
    } else {
        let menu =
            save::SavesMenu { title: "== Load Game ==".to_owned(), saves };

        let chosen = menu_select_with_cancel(&menu, term).await?;
        if let Some(name) = chosen {
            let game = name.load_game().await.prefix(|| {
                format!("Error loading game {}", name.printable())
            })?;
        }
    }
    Ok(())
}

/// Handles when a game is asked to be deleted.
pub async fn delete_game(term: &mut terminal::Handle) -> GameResult<()> {
    let saves = save::list().await?;
    if saves.len() == 0 {
        let dialog = InfoDialog {
            title: "No saved games found",
            message: &format!(
                "No saved games were found at {}",
                save::path()?.display()
            ),
            settings: TextSettings::new().align(1, 2),
        };
        dialog.run(term).await?;
    } else {
        let menu =

            save::SavesMenu { title: "== Delete Game ==".to_owned(), saves };

        let chosen = menu_select_with_cancel(&menu, term).await?;
        if let Some(name) = chosen {
            let prompt = DangerPrompt {
                title: "This cannot be undone, are you sure?".to_owned(),
            };
            let chosen = menu_select(&prompt, term).await?;
            if *chosen == DangerPromptItem::Ok {
                name.delete_game().await.prefix(|| {
                    format!("Error deleting game {}", name.printable())
                })?;
            }
        }
    }
    Ok(())
}
