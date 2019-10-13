pub use std::error::Error;

/// A generic result type.
pub type GameResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
