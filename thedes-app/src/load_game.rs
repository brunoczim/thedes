use std::{
    fmt,
    path::{Path, PathBuf},
};

use thedes_tui::{
    cancellability::Cancellable,
    core::App,
    menu::{self, Menu},
};
use thiserror::Error;
use tokio::{fs, io};

use crate::SAVE_EXTENSION;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read saves from directory {}", path.display())]
    ReadDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to initialize menu")]
    InitMenu(#[source] menu::Error),
    #[error("Failed to run menu")]
    RunMenu(#[source] menu::Error),
}

#[derive(Debug, Clone)]
struct SaveItem {
    name: String,
    path: PathBuf,
}

impl fmt::Display for SaveItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct Component {
    _private: (),
}

impl Component {
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub async fn run(
        &self,
        saves_dir: impl AsRef<Path>,
        app: &mut App,
    ) -> Result<Option<PathBuf>, Error> {
        let saves = self.collect_saves(saves_dir.as_ref()).await?;
        let mut menu = Menu::from_cancellation(
            "% Load Game %",
            saves,
            Cancellable::new(false),
        )
        .map_err(Error::InitMenu)?;

        menu.run(app).await.map_err(Error::RunMenu)?;

        let output = menu.output().map(|item| item.path.clone());
        Ok(output)
    }

    async fn collect_saves(
        &self,
        saves_dir: &Path,
    ) -> Result<Vec<SaveItem>, Error> {
        let mut dir = fs::read_dir(saves_dir).await.map_err(|source| {
            Error::ReadDir { path: saves_dir.to_owned(), source }
        })?;

        let mut items = Vec::<SaveItem>::new();
        while let Some(entry) = dir.next_entry().await.map_err(|source| {
            Error::ReadDir { path: saves_dir.to_owned(), source }
        })? {
            let Some(name) = entry
                .file_name()
                .to_string_lossy()
                .strip_suffix(SAVE_EXTENSION)
                .map(str::to_owned)
            else {
                continue;
            };
            let path = entry.path().to_owned();
            items.push(SaveItem { name, path });
        }

        Ok(items)
    }
}
