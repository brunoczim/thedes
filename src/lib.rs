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

/// Game entity related items.
pub mod entity;

/// Game block related items.
pub mod block;

/// A game session. Loaded from a saved game or a created game.
pub mod session;

use crate::{
    error::{GameResult, ResultExt},
    rand::Seed,
    render::TextSettings,
    session::Session,
    storage::save,
    ui::{DangerPromptItem, InfoDialog, InputDialog, Menu, MenuItem},
};

/// The 'top' function for the game.
pub async fn game_main() -> GameResult<()> {
    let mut term = terminal::Handle::new().await?;

    loop {
        match Menu::MAIN_MENU.select(&mut term).await? {
            MainMenuItem::NewGame => {
                if let Err(err) = new_game(&mut term).await {
                    tracing::error!("{}\n{:?}", err, err.backtrace());
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
                    tracing::error!("{}\n{:?}", err, err.backtrace());
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
                    tracing::error!("{}\n{:?}", err, err.backtrace());
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
        if stem.len() == 0 {
            let dialog = InfoDialog {
                title: "A Save Name Cannot Be Empty",
                message: &"Your input was empty. It cannot be empty for a \
                           save name.",
                settings: TextSettings::new().align(1, 2),
            };
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
    Ok(())
}

/// Handles when a game is asked to be loaded.
pub async fn load_game(term: &mut terminal::Handle) -> GameResult<()> {
    let saves = save::list().await?;
    let menu = Menu { title: "== Load Game ==", items: &saves };
    if let Some(name) = choose_save(term, &menu).await? {
        let game = name
            .load_game()
            .await
            .prefix(|| format!("Error loading game {}", name.printable()))?;
        let mut session = Session::new(game, name.clone())
            .await
            .prefix(|| format!("Error running game {}", name.printable()))?;
        session
            .game_loop(term)
            .await
            .prefix(|| format!("Error running game {}", name.printable()))?;
    }
    Ok(())
}

/// Handles when a game is asked to be deleted.
pub async fn delete_game(term: &mut terminal::Handle) -> GameResult<()> {
    let saves = save::list().await?;
    let menu = Menu { title: "== Delete Game ==", items: &saves };
    if let Some(name) = choose_save(term, &menu).await? {
        let prompt =
            Menu::danger_prompt("This cannot be undone, are you sure?");
        let chosen = prompt.select(term).await?;
        if *chosen == DangerPromptItem::Ok {
            name.delete_game().await.prefix(|| {
                format!("Error deleting game {}", name.printable())
            })?;
        }
    }
    Ok(())
}

/// Asks the user to choose a save given a menu of saves.
pub async fn choose_save<'title, 'items>(
    term: &mut terminal::Handle,
    menu: &Menu<'title, 'items, save::SaveName>,
) -> GameResult<Option<&'items save::SaveName>> {
    if menu.items.len() == 0 {
        let dialog = InfoDialog {
            title: "No saved games found",
            message: &format!(
                "No saved games were found at {}",
                save::path()?.display()
            ),
            settings: TextSettings::new().align(1, 2),
        };
        dialog.run(term).await?;
        Ok(None)
    } else {
        let chosen = menu.select_with_cancel(term).await?;
        Ok(chosen)
    }
}

/// The item of a game's main menu.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MainMenuItem {
    NewGame,
    LoadGame,
    DeleteGame,
    Exit,
}

impl MenuItem for MainMenuItem {
    fn name(&self) -> &str {
        match self {
            MainMenuItem::NewGame => "NEW GAME",
            MainMenuItem::LoadGame => "LOAD GAME",
            MainMenuItem::DeleteGame => "DELETE GAME",
            MainMenuItem::Exit => "EXIT",
        }
    }
}
