use std::{fmt, path::PathBuf};

use thedes_settings::Settings;

pub use thedes_settings::SaveError;
use thedes_tui::{
    cancellability::Cancellable,
    core::App,
    menu::{self, Menu},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to create main settings menu")]
    MainSettingsMenu(#[source] menu::Error),
    #[error("Failed to create audio settings menu")]
    AudioSettingsMenu(#[source] menu::Error),
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error(transparent)]
    LoadSettings(#[from] thedes_settings::LoadError),
    #[error("Failed to initialize component")]
    Init(#[from] InitError),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to run main settings menu")]
    MainSettingsMenu(#[source] menu::Error),
    #[error("Failed to run audio settings menu")]
    AudioSettingsMenu(#[source] menu::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MainSettingsItem {
    Audio,
}

impl fmt::Display for MainSettingsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Audio => "Audio Settings",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum AudioSettingsItem {
    Music,
}

impl fmt::Display for AudioSettingsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Music => "Set Music Volume",
        })
    }
}

#[derive(Debug, Clone)]
pub struct Component {
    main_settings_menu: Menu<MainSettingsItem, Cancellable>,
    audio_settings_menu: Menu<AudioSettingsItem, Cancellable>,
    settings: Settings,
    path: PathBuf,
}

impl Component {
    pub fn new(path: PathBuf, settings: Settings) -> Result<Self, InitError> {
        let main_settings_menu = Menu::from_cancellation(
            "% Settings %",
            [MainSettingsItem::Audio],
            Cancellable::new(false),
        )
        .map_err(InitError::MainSettingsMenu)?;

        let audio_settings_menu = Menu::from_cancellation(
            "((|>  Audio Settings  <|))",
            [AudioSettingsItem::Music],
            Cancellable::new(false),
        )
        .map_err(InitError::AudioSettingsMenu)?;

        Ok(Self { path, settings, main_settings_menu, audio_settings_menu })
    }

    pub async fn load(path: PathBuf) -> Result<Self, LoadError> {
        let settings = Settings::load(&path).await?;
        Ok(Self::new(path, settings)?)
    }

    pub async fn save(&self) -> Result<(), SaveError> {
        self.settings.save(&self.path).await?;
        Ok(())
    }

    pub fn values(&self) -> &Settings {
        &self.settings
    }

    pub fn values_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    pub async fn run(&mut self, app: &mut App) -> Result<(), Error> {
        loop {
            self.main_settings_menu
                .run(app)
                .await
                .map_err(Error::MainSettingsMenu)?;
            match self.main_settings_menu.output() {
                Some(MainSettingsItem::Audio) => {
                    self.audio_settings_menu
                        .run(app)
                        .await
                        .map_err(Error::AudioSettingsMenu)?;
                    match self.audio_settings_menu.output() {
                        Some(AudioSettingsItem::Music) => {},
                        None => (),
                    }
                },
                None => break,
            }
        }

        Ok(())
    }
}
