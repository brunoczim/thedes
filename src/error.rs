use backtrace::Backtrace;
use crossterm::{cursor, style, terminal, Command};
use std::{error::Error as StdError, fmt, ops::Deref, process};

/// Result type used by this library for fallible operations.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Default error type of this library.
#[derive(Debug)]
pub struct Error {
    inner: Box<ErrorInner>,
}

#[derive(Debug)]
struct ErrorInner {
    obj: Box<dyn ErrorExt + Send + Sync>,
    backtrace: Backtrace,
}

impl Error {
    /// The backtrace of this error.
    pub fn backtrace(&self) -> &Backtrace {
        self.inner.obj.backtrace().unwrap_or(&self.inner.backtrace)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.inner.obj)
    }
}

impl<E> From<E> for Error
where
    E: ErrorExt + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        let inner =
            ErrorInner { backtrace: Backtrace::new(), obj: Box::new(error) };
        Self { inner: Box::new(inner) }
    }
}

impl Deref for Error {
    type Target = dyn ErrorExt + Send + Sync;

    fn deref(&self) -> &Self::Target {
        &*self.inner.obj
    }
}

/// Extended error functionality.
pub trait ErrorExt: fmt::Display + fmt::Debug {
    /// Stack backtrace of the error.
    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }
}

impl<E> ErrorExt for E where E: StdError {}

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

impl<D> ErrorExt for PrefixedError<D>
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(self.inner.backtrace())
    }
}

/// Extends result to give an error a prefix.
pub trait ResultExt {
    /// Successful type.
    type Ok;
    /// Adds a prefix to the error message.
    fn prefix<F, D>(self, prefix: F) -> Result<Self::Ok>
    where
        F: FnOnce() -> D,
        D: fmt::Display + fmt::Debug + Send + Sync + 'static;
}

impl<T> ResultExt for Result<T> {
    type Ok = T;
    fn prefix<F, D>(self, prefix: F) -> Result<T>
    where
        F: FnOnce() -> D,
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        self.map_err(|err| {
            PrefixedError { inner: err, prefix: prefix() }.into()
        })
    }
}

impl<T, E> ResultExt for ::std::result::Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    type Ok = T;
    fn prefix<F, D>(self, prefix: F) -> Result<T>
    where
        F: FnOnce() -> D,
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        self.map_err(|err| Error::from(err)).prefix(prefix)
    }
}

/// Checks if the given result is an error. If it is, the process is exited,
/// otherwise, the value stored in Ok is returned.
#[inline]
pub fn exit_on_error<T>(res: Result<T>) -> T {
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
