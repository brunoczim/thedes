use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thedes_audio::AudioControllerType;
use thiserror::Error;
use tokio::{io, task};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    music: u8,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self { music: 125 }
    }
}

impl AudioSettings {
    const VOLUME_STEP: u8 = 15;

    pub fn volume(&self, controller_type: AudioControllerType) -> u8 {
        match controller_type {
            AudioControllerType::Music => self.music,
        }
    }

    pub fn set_volume(
        &mut self,
        controller_type: AudioControllerType,
        value: u8,
    ) {
        match controller_type {
            AudioControllerType::Music => self.music = value,
        }
    }

    pub fn increase_volume(&mut self, controller_type: AudioControllerType) {
        self.set_volume(
            controller_type,
            self.volume(controller_type).saturating_add(Self::VOLUME_STEP),
        );
    }

    pub fn decrease_volume(&mut self, controller_type: AudioControllerType) {
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

impl Default for Settings {
    fn default() -> Self {
        Self { audio: AudioSettings::default() }
    }
}

impl Settings {
    pub async fn load(path: &Path) -> Result<Self, LoadError> {
        task::block_in_place(|| {
            let file =
                File::open(&path).map_err(LoadErrorSource::from).map_err(
                    |source| LoadError { path: path.to_owned(), source },
                )?;
            let mut file = BufReader::new(file);
            serde_json::from_reader(&mut file)
                .map_err(LoadErrorSource::from)
                .map_err(|source| LoadError { path: path.to_owned(), source })
        })
    }

    pub async fn save(&self, path: &Path) -> Result<(), SaveError> {
        task::block_in_place(|| {
            let file =
                File::create(&path).map_err(SaveErrorSource::from).map_err(
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
