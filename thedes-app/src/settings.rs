use std::{fmt, path::PathBuf};

use thedes_settings::{AudioSinkType, Settings};

pub use thedes_settings::SaveError;
use thedes_tui::{
    cancellability::Cancellable,
    core::{
        App,
        audio::{self, Volume, device::SetVolumeError},
    },
    menu::{self, Menu},
    slidebar::{self, Slidebar},
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
    #[error("Failed to set audio output volume")]
    AudioSetVolume(#[from] SetVolumeError),
    #[error("Failed to manipulate slidebar")]
    Slidebar(#[from] slidebar::Error),
    #[error("Failed to save settings")]
    Save(#[from] SaveError),
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
    audio_music_slidebar: Slidebar,
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

        let audio_music_slidebar = Slidebar::new(
            "Set Music Volume",
            slidebar::Config {
                ui_size: 17,
                logical_size: 256,
                logical_current: settings
                    .audio()
                    .volume(AudioSinkType::Music)
                    .into(),
            },
        );

        Ok(Self {
            path,
            settings,
            main_settings_menu,
            audio_settings_menu,
            audio_music_slidebar,
        })
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
                Some(MainSettingsItem::Audio) => loop {
                    self.audio_settings_menu
                        .run(app)
                        .await
                        .map_err(Error::AudioSettingsMenu)?;
                    match self.audio_settings_menu.output() {
                        Some(AudioSettingsItem::Music) => {
                            self.audio_music_slidebar
                                .run(app, |app, current| {
                                    let level = current as Volume;
                                    self.settings.audio_mut().set_volume(
                                        AudioSinkType::Music,
                                        level,
                                    );
                                    app.audio_controller.queue([
                                        audio::Command::new_set_volume(
                                            AudioSinkType::Music,
                                            level,
                                        ),
                                    ]);
                                    _ = app.audio_controller.flush();
                                })
                                .await?;
                        },
                        None => break,
                    }
                },
                None => break,
            }
        }

        self.save().await?;

        Ok(())
    }
}
