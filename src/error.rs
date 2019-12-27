use backtrace::Backtrace;
use std::{error::Error as StdError, fmt, ops::Deref, process};

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

/// Checks if the given result is an error. If it is, the process is exited,
/// otherwise, the value stored in Ok is returned.
#[inline]
pub fn exit_on_error<T>(res: GameResult<T>) -> T {
    match res {
        Ok(val) => val,
        Err(e) => exit_from_error(e),
    }
}

/// A (in theory) rarely called function on the case an error is found on
/// `exit_on_error`.
#[inline(never)]
#[cold]
fn exit_from_error(err: Error) -> ! {
    eprintln!("{}", err);
    tracing::warn!("{}", err);
    tracing::warn!("{:?}", err.backtrace());
    process::exit(-1);
}
