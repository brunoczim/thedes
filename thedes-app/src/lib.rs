use std::path::PathBuf;

use thiserror::Error;

pub mod root;
pub mod load_game;
pub mod session;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    InitRoot(#[from] root::InitError),
    #[error(transparent)]
    RunRoot(#[from] root::Error),
}

pub async fn run(
    saves_dir: PathBuf,
    mut app: thedes_tui::core::App,
) -> Result<(), Error> {
    root::Component::new(saves_dir)?.run(&mut app).await?;
    Ok(())
}
