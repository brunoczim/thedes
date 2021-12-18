use crate::{language, npc, player, thede};
use backtrace::Backtrace;
use std::{error::Error as StdError, fmt};

pub type Result<T> = std::result::Result<T, Error>;

pub trait ResultExt {
    type Ok;

    fn erase_err(self) -> Result<Self::Ok>;
}

impl<T, E> ResultExt for std::result::Result<T, E>
where
    E: StdError,
{
    type Ok = T;

    fn erase_err(self) -> Result<<Self as ResultExt>::Ok> {
        self.map_err(Error::erase)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    #[inline]
    pub fn new(error_kind: ErrorKind) -> Self {
        Self {
            inner: Box::new(ErrorInner {
                backtrace: Backtrace::new(),
                kind: error_kind,
            }),
        }
    }

    #[inline]
    pub fn erase<E>(error: E) -> Self
    where
        E: StdError,
    {
        Self::new(ErrorKind::erase(error))
    }

    #[inline]
    pub fn backtrace(&self) -> &Backtrace {
        &self.inner.backtrace
    }

    #[inline]
    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{}\n{:#?}", self.kind(), self.backtrace())
    }
}

impl StdError for Error {}

impl<E> From<E> for Error
where
    ErrorKind: From<E>,
{
    #[inline]
    fn from(error_kind: E) -> Self {
        Self::new(error_kind.into())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ErrorInner {
    backtrace: Backtrace,
    kind: ErrorKind,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ErrorKind {
    BadPlayerId(BadPlayerId),
    BadNpcId(BadNpcId),
    BadThedeId(BadThedeId),
    BadLanguageId(BadLanguageId),
    BadSeedString(BadSeedString),
    CustomError(CustomError),
}

impl ErrorKind {
    #[inline]
    pub fn erase<E>(error: E) -> Self
    where
        E: StdError,
    {
        ErrorKind::CustomError(CustomError::erase(error))
    }

    #[inline]
    pub fn as_dyn(&self) -> &(dyn StdError + Send + Sync) {
        match self {
            ErrorKind::BadPlayerId(error) => error,
            ErrorKind::BadNpcId(error) => error,
            ErrorKind::BadThedeId(error) => error,
            ErrorKind::BadLanguageId(error) => error,
            ErrorKind::BadSeedString(error) => error,
            ErrorKind::CustomError(error) => error,
        }
    }
}

impl fmt::Display for ErrorKind {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_dyn(), fmtr)
    }
}

impl StdError for ErrorKind {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BadPlayerId {
    pub id: player::Id,
}

impl fmt::Display for BadPlayerId {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "invalid player ID {}", self.id)
    }
}

impl StdError for BadPlayerId {}

impl From<BadPlayerId> for ErrorKind {
    #[inline]
    fn from(error: BadPlayerId) -> Self {
        ErrorKind::BadPlayerId(error)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BadNpcId {
    pub id: npc::Id,
}

impl fmt::Display for BadNpcId {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "invalid npc ID {}", self.id)
    }
}

impl StdError for BadNpcId {}

impl From<BadNpcId> for ErrorKind {
    #[inline]
    fn from(error: BadNpcId) -> Self {
        ErrorKind::BadNpcId(error)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BadThedeId {
    pub id: thede::Id,
}

impl fmt::Display for BadThedeId {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "invalid thede ID {}", self.id)
    }
}

impl StdError for BadThedeId {}

impl From<BadThedeId> for ErrorKind {
    #[inline]
    fn from(error: BadThedeId) -> Self {
        ErrorKind::BadThedeId(error)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BadLanguageId {
    pub id: language::Id,
}

impl fmt::Display for BadLanguageId {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "invalid language ID {}", self.id)
    }
}

impl StdError for BadLanguageId {}

impl From<BadLanguageId> for ErrorKind {
    #[inline]
    fn from(error: BadLanguageId) -> Self {
        ErrorKind::BadLanguageId(error)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BadSeedString;

impl fmt::Display for BadSeedString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad("Seed is not a 16-digit hexadecimal number")
    }
}

impl StdError for BadSeedString {}

impl From<BadSeedString> for ErrorKind {
    #[inline]
    fn from(error: BadSeedString) -> Self {
        ErrorKind::BadSeedString(error)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomError {
    pub message: String,
}

impl CustomError {
    #[inline]
    fn erase<E>(error: E) -> Self
    where
        E: StdError,
    {
        Self { message: error.to_string() }
    }
}

impl fmt::Display for CustomError {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{}", self.message)
    }
}

impl StdError for CustomError {}

impl From<CustomError> for ErrorKind {
    #[inline]
    fn from(error: CustomError) -> Self {
        ErrorKind::CustomError(error)
    }
}
