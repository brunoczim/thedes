use std::path::PathBuf;

use thiserror::Error;

pub mod root;
pub mod session;
pub mod settings;

pub const SAVE_EXTENSION: &'static str = ".save.thedes";

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    InitRoot(#[from] root::InitError),
    #[error(transparent)]
    RunRoot(#[from] root::Error),
}

#[derive(Debug, Clone)]
pub struct Config {
    saves_dir: PathBuf,
    settings_path: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        Self {
            saves_dir: PathBuf::from("."),
            settings_path: PathBuf::from("thedes-settings.json"),
        }
    }

    pub fn with_saves_dir(self, saves_dir: PathBuf) -> Self {
        Self { saves_dir, ..self }
    }

    pub fn with_settings_path(self, settings_path: PathBuf) -> Self {
        Self { settings_path, ..self }
    }

    pub async fn run(
        self,
        mut app: thedes_tui::core::App,
    ) -> Result<(), Error> {
        root::Component::new(root::Config {
            saves_dir: self.saves_dir,
            settings_path: self.settings_path,
        })?
        .run(&mut app)
        .await?;
        Ok(())
    }
}
