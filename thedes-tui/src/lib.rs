use screen::RenderError;
use thiserror::Error;
use tokio::task::JoinError;

pub mod mutation;
pub mod geometry;
pub mod color;
pub mod grapheme;
pub mod tile;
pub mod screen;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to render to the screen")]
    Render(
        #[from]
        #[source]
        RenderError,
    ),
    #[error("Failed to join task")]
    Join(
        #[from]
        #[source]
        JoinError,
    ),
}
