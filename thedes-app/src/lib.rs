use std::fmt;

use thedes_tui::{
    core::event::Key,
    input::{self, Input},
    menu::{self, Menu},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to render TUI")]
    RenderText(
        #[from]
        #[source]
        thedes_tui::text::Error,
    ),
    #[error("Failed to interact with screen canvas")]
    CanvasFlush(
        #[from]
        #[source]
        thedes_tui::core::screen::FlushError,
    ),
    #[error("Failed to run main menu")]
    MainMenu(
        #[from]
        #[source]
        menu::Error,
    ),
    #[error("Failed to run input")]
    SeedInput(
        #[from]
        #[source]
        input::Error,
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

pub async fn root(mut app: thedes_tui::core::App) -> Result<(), Error> {
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

    let mut main_menu = Menu::new("=== T H E D E S ===", &main_menu_items)?
        .with_keybindings(main_menu_bindings);

    let mut seed_input = Input::new(input::Config {
        max: 32,
        filter: |ch: char| ch.is_ascii_hexdigit(),
        title: "New World Seed",
    })?;

    loop {
        main_menu.run(&mut app).await?;

        match main_menu.output() {
            MainMenuItem::NewGame => {
                seed_input.run(&mut app).await?;
            },
            MainMenuItem::LoadGame => {},
            MainMenuItem::Settings => {},
            MainMenuItem::Quit => break,
        }
    }

    Ok(())
}
