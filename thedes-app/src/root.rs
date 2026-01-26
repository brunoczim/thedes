use std::{fmt, path::PathBuf};

use thedes_asset::Assets;
use thedes_audio::AudioClient;
use thedes_tui::{
    core::event::Key,
    menu::{self, Menu},
};
use thiserror::Error;

use crate::{SAVE_EXTENSION, load_game, session};

pub mod new_game;
pub mod game_creation;

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
    #[error("Failed to connect audio controller")]
    Audio(#[from] thedes_audio::ConnectError),
}

#[derive(Debug, Error)]
pub enum Error {
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
        new_game::Error,
    ),
    #[error("Failed to run game creation")]
    GameCreation(
        #[from]
        #[source]
        game_creation::Error,
    ),
    #[error("Failed to run game session component")]
    Session(
        #[from]
        #[source]
        session::Error,
    ),
    #[error("Failed to create a game session")]
    SessionInit(
        #[from]
        #[source]
        session::InitError,
    ),
    #[error("Failed to load game")]
    LoadGame(#[from] load_game::Error),
    #[error("Failed to load asset")]
    LoadAsset(#[from] thedes_asset::LoadError),
    #[error("Failed to play audio")]
    AudioPlay(#[from] thedes_audio::PlayNowError),
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

// #[derive(Debug, Clone)]
pub struct Component {
    main_menu: Menu<MainMenuItem>,
    new_game: new_game::Component,
    game_creation: game_creation::Component,
    load_game: load_game::Component,
    session_config: session::Config,
    saves_dir: PathBuf,
    audio_client: AudioClient,
}

impl Component {
    pub fn new(saves_dir: PathBuf) -> Result<Self, InitError> {
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
        let game_creation = game_creation::Component::new();

        let load_game = load_game::Component::new();

        let audio_client = AudioClient::connect()?;

        Ok(Self {
            main_menu,
            new_game,
            game_creation,
            load_game,
            session_config: session::Config::new(),
            saves_dir: saves_dir.into(),
            audio_client,
        })
    }

    pub async fn run(
        &mut self,
        app: &mut thedes_tui::core::App,
    ) -> Result<(), Error> {
        let assets = Assets::get().await?;
        self.audio_client.play_now(&assets.sound.main_theme[..])?;

        loop {
            self.main_menu.run(app).await?;

            match self.main_menu.output() {
                MainMenuItem::NewGame => {
                    self.new_game.set_seed(rand::random());
                    self.new_game.run(app).await?;
                    let seed = self.new_game.form().seed;
                    let config = thedes_gen::Config::new().with_seed(seed);
                    if let Some(game) =
                        self.game_creation.run(app, config).await?
                    {
                        let mut save_path = self.saves_dir.clone();
                        save_path.push(format!(
                            "{}{}",
                            self.new_game.form().name,
                            SAVE_EXTENSION,
                        ));
                        let mut session = self
                            .session_config
                            .clone()
                            .finish(save_path, game)?;
                        session.run(app).await?;
                    }
                },

                MainMenuItem::LoadGame => {
                    if let Some(save_path) =
                        self.load_game.run(&self.saves_dir, app).await?
                    {
                        let mut session = self
                            .session_config
                            .clone()
                            .finish_loading(save_path)
                            .await?;
                        session.run(app).await?;
                    }
                },
                MainMenuItem::Settings => {},
                MainMenuItem::Quit => break,
            }
        }

        Ok(())
    }
}
