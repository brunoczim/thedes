use std::{
    borrow::Cow,
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thedes_tui_core::audio::Volume;
use thiserror::Error;
use tokio::{io, task};
use tracing::Level;

#[derive(Debug, Error)]
pub enum LoadErrorSource {
    #[error("I/O error happened")]
    Io(#[from] io::Error),
    #[error("Failed to deserialize")]
    Deserialize(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
#[error("Failed to load game from {path}")]
pub struct LoadError {
    pub path: PathBuf,
    #[source]
    pub source: LoadErrorSource,
}

#[derive(Debug, Error)]
pub enum SaveErrorSource {
    #[error("I/O error happened")]
    Io(#[from] io::Error),
    #[error("Failed to serialize")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
#[error("Failed to save game to {path}")]
pub struct SaveError {
    pub path: PathBuf,
    #[source]
    pub source: SaveErrorSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AudioSinkType {
    Music,
    Fx,
}

impl AudioSinkType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Music => "Music",
            Self::Fx => "FX",
        }
    }
}

impl From<AudioSinkType> for &'static str {
    fn from(value: AudioSinkType) -> Self {
        value.name()
    }
}

impl From<AudioSinkType> for Cow<'static, str> {
    fn from(value: AudioSinkType) -> Self {
        <&'static str>::from(value).into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    music: Volume,
    fx: Volume,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self { music: 125, fx: 190 }
    }
}

impl AudioSettings {
    const VOLUME_STEP: u8 = 15;

    pub fn volume(&self, controller_type: AudioSinkType) -> u8 {
        match controller_type {
            AudioSinkType::Music => self.music,
            AudioSinkType::Fx => self.fx,
        }
    }

    pub fn set_volume(&mut self, controller_type: AudioSinkType, value: u8) {
        match controller_type {
            AudioSinkType::Music => self.music = value,
            AudioSinkType::Fx => self.fx = value,
        }
    }

    pub fn increase_volume(&mut self, controller_type: AudioSinkType) {
        self.set_volume(
            controller_type,
            self.volume(controller_type).saturating_add(Self::VOLUME_STEP),
        );
    }

    pub fn decrease_volume(&mut self, controller_type: AudioSinkType) {
        self.set_volume(
            controller_type,
            self.volume(controller_type).saturating_sub(Self::VOLUME_STEP),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    audio: AudioSettings,
}

#[expect(clippy::derivable_impls)]
impl Default for Settings {
    fn default() -> Self {
        Self { audio: AudioSettings::default() }
    }
}

impl Settings {
    pub async fn load(path: &Path) -> Result<Self, LoadError> {
        if tracing::event_enabled!(Level::DEBUG) {
            tracing::debug!(
                path = path.display().to_string(),
                "Loading settings"
            );
        }
        task::block_in_place(|| {
            let file =
                File::open(path).map_err(LoadErrorSource::from).map_err(
                    |source| LoadError { path: path.to_owned(), source },
                )?;
            let mut file = BufReader::new(file);
            serde_json::from_reader(&mut file)
                .map_err(LoadErrorSource::from)
                .map_err(|source| LoadError { path: path.to_owned(), source })
        })
    }

    pub async fn load_or_default(path: &Path) -> Self {
        match Self::load(path).await {
            Ok(this) => this,
            Err(e) => {
                let mut chain = String::new();
                let mut next = Some(&e as &(dyn Error + 'static));
                while let Some(current) = next {
                    chain.push_str(&current.to_string());
                    chain.push('\n');
                    next = current.source();
                }
                let path = path.display().to_string();
                tracing::error!(
                    %chain,
                    %path,
                    "Failed to load configuration",
                );
                Self::default()
            },
        }
    }

    pub async fn save(&self, path: &Path) -> Result<(), SaveError> {
        if tracing::event_enabled!(Level::DEBUG) {
            tracing::debug!(
                path = path.display().to_string(),
                "Saving settings"
            );
        }
        task::block_in_place(|| {
            let file =
                File::create(path).map_err(SaveErrorSource::from).map_err(
                    |source| SaveError { path: path.to_owned(), source },
                )?;
            let mut file = BufWriter::new(file);
            serde_json::to_writer(&mut file, self)
                .map_err(SaveErrorSource::from)
                .map_err(|source| SaveError {
                    path: path.to_owned(),
                    source,
                })?;
            Ok(())
        })
    }

    pub fn audio(&self) -> &AudioSettings {
        &self.audio
    }

    pub fn audio_mut(&mut self) -> &mut AudioSettings {
        &mut self.audio
    }
}
