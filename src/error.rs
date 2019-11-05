use backtrace::Backtrace;
use std::{error::Error as StdError, fmt, ops::Deref};

/// A generic result type.
pub type GameResult<T> = Result<T, Error>;

/// A generic error.
#[derive(Debug)]
pub struct Error {
    backtrace: Backtrace,
    obj: Box<dyn StdError + Send + Sync>,
}

impl Error {
    /// The backtrace of this error.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.obj)
    }
}

impl<E> From<E> for Error
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Self { backtrace: Backtrace::new(), obj: error.into() }
    }
}

impl Deref for Error {
    type Target = dyn StdError + Send + Sync;

    fn deref(&self) -> &Self::Target {
        &*self.obj
    }
}
