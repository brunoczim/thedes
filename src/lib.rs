#![deny(unused_must_use)]

/// Exports macros.
#[macro_use]
pub mod macros;

/// Exports error utilites.
pub mod error;

/// Contains mathematics related utilities, such as random number generator and
/// plane types.
pub mod math;

/// Exports graphics related utilites.
pub mod graphics;

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

/// Game generated structures.
pub mod structures;

/// A game session. Loaded from a saved game or a created game.
pub mod session;

use crate::{
    error::{Result, ResultExt},
    graphics::{Color, ColoredGString, ColorsKind, GString, Style},
    math::rand::Seed,
    session::Session,
    storage::save::{self, SaveName},
    ui::{
        DangerPromptOption,
        InfoDialog,
        InputDialog,
        LabeledOption,
        Labels,
        Menu,
    },
};
use std::str::FromStr;

/// Game app's start point.
pub async fn game_main(term: terminal::Handle) -> Result<()> {
    let main_menu = MainMenuOption::menu();

    loop {
        let opt = main_menu.select(&term).await?;
        let res = match &main_menu.options[opt] {
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
        colored_gstring![(gstring!["== New Game  =="], ColorsKind::default())],
        String::new(),
        term,
        save::MAX_NAME,
        save::is_valid_name_char,
    );

    let maybe_stem = loop {
        match dialog.select_with_cancel().await? {
            Some(stem) => {
                if stem.len() > 0 {
                    break Some(stem);
                }

                let dialog = InfoDialog::new(
                    gstring!["A Save Name Cannot Be Empty"],
                    gstring![
                        "Your input was empty. It cannot be empty for a save \
                         name."
                    ],
                );
                dialog.run(term).await?;
            },
            None => break None,
        }
    };

    if let Some(stem) = maybe_stem {
        let save_name = save::SaveName::from_stem(stem).await?;
        NewGameMenu::new(save_name).execute(term).await?;
    }
    Ok(())
}

/// An option of the game's main menu.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum NewGameOption {
    /// User asked to actually create the world.
    Create,
    /// User asked to set the seed.
    SetSeed,
}

impl LabeledOption for NewGameOption {
    fn label(&self) -> ColoredGString<ColorsKind> {
        let string = match self {
            NewGameOption::Create => "DONE! CREATE",
            NewGameOption::SetSeed => "SET SEED",
        };
        colored_gstring![(gstring![string], ColorsKind::default())]
    }
}

/// A new game menu, after the new world has been requested.
#[derive(Debug, Clone)]
struct NewGameMenu {
    save_name: SaveName,
    seed: Option<Seed>,
    ui: Menu<NewGameOption>,
}

impl NewGameMenu {
    fn new(save_name: SaveName) -> Self {
        Self {
            ui: Menu::new(
                gstring![format!("<* Create {} *>", save_name.printable())],
                vec![NewGameOption::Create, NewGameOption::SetSeed],
            ),
            save_name,
            seed: None,
        }
    }

    async fn execute(mut self, term: &terminal::Handle) -> Result<()> {
        loop {
            let chosen = self.ui.select_with_cancel(term).await?;
            match chosen.map(|index| &self.ui.options[index]) {
                None => break Ok(()),

                Some(NewGameOption::Create) => {
                    self.execute_create(term).await?;
                    break Ok(());
                },

                Some(NewGameOption::SetSeed) => {
                    self.execute_set_seed(term).await?
                },
            }
        }
    }

    async fn execute_create(self, term: &terminal::Handle) -> Result<()> {
        write_loading(&mut term.lock_screen().await)?;
        let game = self
            .save_name
            .new_game(self.seed.unwrap_or(Seed::random()))
            .await
            .prefix(|| {
                format!("Error creating game {}", self.save_name.printable())
            })?;

        let mut session =
            Session::new(game, self.save_name.clone()).await.prefix(|| {
                format!("Error running game {}", self.save_name.printable())
            })?;

        session.game_loop(term).await.prefix(|| {
            format!("Error running game {}", self.save_name.printable())
        })?;
        Ok(())
    }

    async fn execute_set_seed(
        &mut self,
        term: &terminal::Handle,
    ) -> Result<()> {
        let mut dialog = InputDialog::new(
            gstring!["··· Set Seed (HexaDecimal) ···"],
            self.seed.map(|seed| seed.to_string()).unwrap_or(String::new()),
            term,
            16,
            |ch| ch.is_ascii_hexdigit(),
        );

        match dialog.select_with_cancel().await? {
            Some(digits) => self.seed = Seed::from_str(&digits).ok(),
            None => (),
        }
        Ok(())
    }
}

/// Handles when a game is asked to be loaded.
pub async fn load_game(term: &terminal::Handle) -> Result<()> {
    let saves = save::list().await?;
    let menu = Menu::new(gstring!["== Load Game =="], saves);
    if let Some(name) = choose_save(term, &menu).await? {
        write_loading(&mut term.lock_screen().await)?;
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
    let menu = Menu::new(
        colored_gstring![(
            gstring!["== Delete Game =="],
            ColorsKind::default()
        )],
        saves,
    );
    if let Some(name) = choose_save(term, &menu).await? {
        let prompt = DangerPromptOption::menu(gstring![
            "This cannot be undone, are you sure?"
        ]);
        let chosen = prompt
            .select(
                term,
                |screen| async move { Ok(screen.clear(Color::Black)) },
            )
            .await?;
        if prompt.options[chosen] == DangerPromptOption::Ok {
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
        dialog
            .run(term, |screen| async move { Ok(screen.clear(Color::Black)) })
            .await?;
        Ok(None)
    } else {
        let chosen = menu
            .select_with_cancel(
                term,
                &Labels {
                    unselected: colored_gstring![(
                        gstring!["CANCEL"],
                        ColorsKind::default()
                    )],
                    selected: colored_gstring![(
                        gstring!["CANCEL"],
                        !ColorsKind::default()
                    )],
                },
                |screen| async move { Ok(screen.clear(Color::Black)) },
            )
            .await?;
        Ok(chosen.map(|i| &menu.options[i]))
    }
}

/// An option of the game's main menu.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MainMenuOption {
    /// User asked for a new game.
    NewGame,
    /// User asked to load a game.
    LoadGame,
    /// User asked to delete a game.
    DeleteGame,
    /// User asked to leave.
    Exit,
}

impl MainMenuOption {
    fn menu() -> Menu<Self> {
        Menu::new(
            colored_gstring![(
                gstring!["=== T H E D E S ==="],
                ColorsKind::default()
            )],
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
    fn label(&self) -> GString {
        let string = match self {
            MainMenuOption::NewGame => "NEW GAME",
            MainMenuOption::LoadGame => "LOAD GAME",
            MainMenuOption::DeleteGame => "DELETE GAME",
            MainMenuOption::Exit => "EXIT",
        };
        colored_gstring![(gstring![string], ColorsKind::default())]
    }
}

/// Shows a loading screen to the user.
pub fn write_loading(screen: &mut terminal::Screen) -> Result<()> {
    screen.clear(Color::Black);
    let style = Style::new()
        .top_margin(screen.handle().screen_size().y / 3)
        .align(1, 2);
    screen.styled_text(&gstring!["Loading..."], style)?;
    Ok(())
}
