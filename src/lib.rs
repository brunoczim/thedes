/// Exports macros.
#[macro_use]
pub mod macros;

/// Exports error utilites.
pub mod error;

/// Random number generation utilites.
pub mod rand;

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

/// Game matter: things that have only a physical form.
pub mod matter;

/// Game entities: things that have a non-physical form.
pub mod entity;

/// A game session. Loaded from a saved game or a created game.
pub mod session;

use crate::{
    error::{Result, ResultExt},
    graphics::GString,
    rand::Seed,
    session::Session,
    storage::save,
    ui::{DangerPromptOption, InfoDialog, InputDialog, Menu, MenuOption},
};

/// Game app's start point.
pub async fn game_main(term: terminal::Handle) -> Result<()> {
    let main_menu = MainMenuOption::menu();

    loop {
        let res = match main_menu.select(&term).await? {
            MainMenuOption::NewGame => new_game(&term).await,
            MainMenuOption::LoadGame => load_game(&term).await,
            MainMenuOption::DeleteGame => delete_game(&term).await,
            MainMenuOption::Exit => break,
        };

        if let Err(err) = res {
            tracing::error!("{}\n{:?}", err, err.backtrace());
            let dialog = InfoDialog::new(
                gstring!["Error"],
                gstring![format!("{}", err)],
            );
            dialog.run(&term).await?;
        }
    }

    Ok(())
}

/// Handles when a new game is asked.
pub async fn new_game(term: &terminal::Handle) -> Result<()> {
    let mut dialog = InputDialog::new(
        gstring!["== New Game  =="],
        String::new(),
        term,
        save::MAX_NAME,
        save::is_valid_name_char,
    );
    let input = dialog.select_with_cancel().await?;

    if let Some(stem) = input {
        if stem.len() == 0 {
            let dialog = InfoDialog::new(
                gstring!["A Save Name Cannot Be Empty"],
                gstring![
                    "Your input was empty. It cannot be empty for a save name."
                ],
            );
            dialog.run(term).await?;
        } else {
            let save_name = save::SaveName::from_stem(stem).await?;
            let game =
                save_name.new_game(Seed::random()).await.prefix(|| {
                    format!("Error creating game {}", save_name.name())
                })?;

            let mut session =
                Session::new(game, save_name.clone()).await.prefix(|| {
                    format!("Error running game {}", save_name.printable())
                })?;
            session.game_loop(term).await.prefix(|| {
                format!("Error running game {}", save_name.printable())
            })?;
        }
    }
    Ok(())
}

/// Handles when a game is asked to be loaded.
pub async fn load_game(term: &terminal::Handle) -> Result<()> {
    let saves = save::list().await?;
    let menu = Menu::new(gstring!["== Load Game =="], saves);
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
pub async fn delete_game(term: &terminal::Handle) -> Result<()> {
    let saves = save::list().await?;
    let menu = Menu::new(gstring!["== Delete Game =="], saves);
    if let Some(name) = choose_save(term, &menu).await? {
        let prompt = DangerPromptOption::menu(gstring![
            "This cannot be undone, are you sure?"
        ]);
        let chosen = prompt.select(term).await?;
        if *chosen == DangerPromptOption::Ok {
            name.delete_game().await.prefix(|| {
                format!("Error deleting game {}", name.printable())
            })?;
        }
    }
    Ok(())
}

/// Asks the user to choose a save given a menu of saves.
pub async fn choose_save<'menu>(
    term: &terminal::Handle,
    menu: &'menu Menu<save::SaveName>,
) -> Result<Option<&'menu save::SaveName>> {
    if menu.options.len() == 0 {
        let dialog = InfoDialog::new(
            gstring!["No saved games found"],
            gstring![format!(
                "No saved games were found at {}",
                save::path()?.display()
            )],
        );
        dialog.run(term).await?;
        Ok(None)
    } else {
        let chosen = menu.select_with_cancel(term).await?;
        Ok(chosen)
    }
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
            gstring!["=== T H E D E S ==="],
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
    fn name(&self) -> GString {
        let string = match self {
            MainMenuOption::NewGame => "NEW GAME",
            MainMenuOption::LoadGame => "LOAD GAME",
            MainMenuOption::DeleteGame => "DELETE GAME",
            MainMenuOption::Exit => "EXIT",
        };
        gstring![string]
    }
}
