use std::fmt;

use thedes_domain::{
    game::{self, Game},
    geometry::{CoordPair, Rect},
    map::{self, Map},
    player::{self, PlayerPosition},
};
use thedes_geometry::orientation::Direction;
use thedes_tui::{
    core::event::Key,
    menu::{self, Menu},
};
use thiserror::Error;

use crate::session;

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
    #[error("Inconsistent main menu, missing quit")]
    MissingQuit,
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
    #[error("Failed to run game session component")]
    Session(
        #[from]
        #[source]
        session::RunError,
    ),
    #[error("Failed to create a game")]
    GameInit(
        #[from]
        #[source]
        game::InitError,
    ),
    #[error("Failed to create a game map")]
    MapInit(
        #[from]
        #[source]
        map::InitError,
    ),
    #[error("Failed to create a game map")]
    PlayerPositionInit(
        #[from]
        #[source]
        player::InitError,
    ),
    #[error("Failed to create a game session")]
    SessionInit(
        #[from]
        #[source]
        session::InitError,
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
    session_config: session::Config,
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
            .ok_or(InitError::MissingQuit)?;

        let main_menu_bindings = menu::default_key_bindings()
            .with(Key::Char('q'), menu::Command::SelectConfirm(quit_position));

        let main_menu = Menu::new("=== T H E D E S ===", main_menu_items)?
            .with_keybindings(main_menu_bindings);

        let new_game = new_game::Component::new()?;

        Ok(Self { main_menu, new_game, session_config: session::Config::new() })
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
                    let map = Map::new(Rect {
                        top_left: CoordPair { x: 0, y: 0 },
                        size: CoordPair { x: 1000, y: 1000 },
                    })?;
                    let player_position = PlayerPosition::new(
                        CoordPair { y: 50, x: 50 },
                        Direction::Up,
                    )?;
                    let game = Game::new(map, player_position)?;
                    let mut session =
                        self.session_config.clone().finish(game)?;
                    session.run(app).await?;
                },

                MainMenuItem::LoadGame => {},
                MainMenuItem::Settings => {},
                MainMenuItem::Quit => break,
            }
        }

        Ok(())
    }
}
