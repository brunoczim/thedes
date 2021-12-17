use backtrace::Backtrace;
use std::{error::Error as StdError, fmt};
use thedes_common::{language, npc, player, thede};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    pub fn new(error_kind: ErrorKind) -> Self {
        Self {
            inner: Box::new(ErrorInner {
                backtrace: Backtrace::new(),
                kind: error_kind,
            }),
        }
    }

    pub fn erase<E>(error: E) -> Self
    where
        E: StdError,
    {
        Self::new(ErrorKind::erase(error))
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.inner.backtrace
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{}\n{:#?}", self.kind(), self.backtrace())
    }
}

impl StdError for Error {}

impl<E> From<E> for Error
where
    ErrorKind: From<E>,
{
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
    CustomError(CustomError),
}

impl ErrorKind {
    pub fn erase<E>(error: E) -> Self
    where
        E: StdError,
    {
        ErrorKind::CustomError(CustomError::erase(error))
    }

    pub fn as_dyn(&self) -> &(dyn StdError + Send + Sync) {
        match self {
            ErrorKind::BadPlayerId(error) => error,
            ErrorKind::BadNpcId(error) => error,
            ErrorKind::BadThedeId(error) => error,
            ErrorKind::BadLanguageId(error) => error,
            ErrorKind::CustomError(error) => error,
        }
    }
}

impl fmt::Display for ErrorKind {
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
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "invalid player ID {}", self.id)
    }
}

impl StdError for BadPlayerId {}

impl From<BadPlayerId> for ErrorKind {
    fn from(error: BadPlayerId) -> Self {
        ErrorKind::BadPlayerId(error)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BadNpcId {
    pub id: npc::Id,
}

impl fmt::Display for BadNpcId {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "invalid npc ID {}", self.id)
    }
}

impl StdError for BadNpcId {}

impl From<BadNpcId> for ErrorKind {
    fn from(error: BadNpcId) -> Self {
        ErrorKind::BadNpcId(error)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BadThedeId {
    pub id: thede::Id,
}

impl fmt::Display for BadThedeId {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "invalid thede ID {}", self.id)
    }
}

impl StdError for BadThedeId {}

impl From<BadThedeId> for ErrorKind {
    fn from(error: BadThedeId) -> Self {
        ErrorKind::BadThedeId(error)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BadLanguageId {
    pub id: language::Id,
}

impl fmt::Display for BadLanguageId {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "invalid language ID {}", self.id)
    }
}

impl StdError for BadLanguageId {}

impl From<BadLanguageId> for ErrorKind {
    fn from(error: BadLanguageId) -> Self {
        ErrorKind::BadLanguageId(error)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomError {
    pub message: String,
}

impl CustomError {
    fn erase<E>(error: E) -> Self
    where
        E: StdError,
    {
        Self { message: error.to_string() }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{}", self.message)
    }
}

impl StdError for CustomError {}

impl From<CustomError> for ErrorKind {
    fn from(error: CustomError) -> Self {
        ErrorKind::CustomError(error)
    }
}
