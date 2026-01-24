use std::path::PathBuf;

use thiserror::Error;

pub mod root;
pub mod load_game;
pub mod session;

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
}

impl Config {
    pub fn new() -> Self {
        Self { saves_dir: PathBuf::from(".") }
    }

    pub fn with_saves_dir(self, saves_dir: PathBuf) -> Self {
        Self { saves_dir }
    }

    pub async fn run(
        self,
        mut app: thedes_tui::core::App,
    ) -> Result<(), Error> {
        root::Component::new(self.saves_dir)?.run(&mut app).await?;
        Ok(())
    }
}
