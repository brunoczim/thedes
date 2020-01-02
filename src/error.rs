use backtrace::Backtrace;
use crossterm::{cursor, style, terminal, Command};
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
    // We're exiting below, so, no problem blocking.
    restore_term();
    eprintln!("{}", err);
    tracing::warn!("{}", err);
    tracing::warn!("{:?}", err.backtrace());
    process::exit(-1);
}

#[cfg(windows)]
/// Best-effort function.
pub fn restore_term() {
    let _ = terminal::disable_raw_mode();
    print!("{}", cursor::Show);
    print!("{}", style::SetBackgroundColor(style::Color::Reset));
    print!("{}", style::SetForegroundColor(style::Color::Reset));
    if terminal::LeaveAlternateScreen.is_ansi_code_supported() {
        print!("{}", terminal::LeaveAlternateScreen.ansi_code());
    }
    println!();
}

#[cfg(unix)]
/// Best-effort function.
pub fn restore_term() {
    let _ = terminal::disable_raw_mode();
    print!("{}", cursor::Show);
    print!("{}", style::SetBackgroundColor(style::Color::Reset));
    print!("{}", style::SetForegroundColor(style::Color::Reset));
    println!("{}", terminal::LeaveAlternateScreen.ansi_code());
}

#[derive(Debug)]
/// An errror with a prefixed message.
struct PrefixedError<D>
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    prefix: D,
    inner: Error,
}

impl<D> fmt::Display for PrefixedError<D>
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: {}", self.prefix, self.inner)
    }
}

impl<D> StdError for PrefixedError<D> where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static
{
}

pub trait ResultExt {
    type Ok;
    fn prefix<F, D>(self, prefix: F) -> GameResult<Self::Ok>
    where
        F: FnOnce() -> D,
        D: fmt::Display + fmt::Debug + Send + Sync + 'static;
}

impl<T> ResultExt for GameResult<T> {
    type Ok = T;
    fn prefix<F, D>(self, prefix: F) -> GameResult<T>
    where
        F: FnOnce() -> D,
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        self.map_err(|err| {
            PrefixedError { inner: err, prefix: prefix() }.into()
        })
    }
}

impl<T, E> ResultExt for Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    type Ok = T;
    fn prefix<F, D>(self, prefix: F) -> GameResult<T>
    where
        F: FnOnce() -> D,
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        self.map_err(|err| Error::from(err)).prefix(prefix)
    }
}
