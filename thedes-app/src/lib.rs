use thiserror::Error;

pub mod root;
pub mod session;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    InitRoot(#[from] root::InitError),
    #[error(transparent)]
    RunRoot(#[from] root::Error),
}

pub async fn run(mut app: thedes_tui::core::App) -> Result<(), Error> {
    root::Component::new()?.run(&mut app).await?;
    Ok(())
}
