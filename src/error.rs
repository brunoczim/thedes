pub use std::error::Error;

/// A generic result type.
pub type Result<T> = ::std::result::Result<T, Box<dyn Error>>;
