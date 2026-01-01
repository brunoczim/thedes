use std::{
    fmt,
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub struct Error {
    path: Option<PathBuf>,
    #[source]
    kind: ErrorKind,
}

impl Error {
    pub(crate) fn new(kind: impl Into<ErrorKind>) -> Self {
        Self { path: None, kind: kind.into() }
    }

    pub(crate) fn new_with_path<E>(
        path: impl Into<PathBuf>,
    ) -> impl FnOnce(E) -> Self
    where
        E: Into<ErrorKind>,
    {
        |kind| Self::with_path(path)(Self::new(kind))
    }

    pub(crate) fn with_path(
        path: impl Into<PathBuf>,
    ) -> impl FnOnce(Self) -> Self {
        |this| Self { path: Some(path.into()), ..this }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(path) = &self.path {
            write!(f, ", path: {}", path.display())?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Failed reading development scripts file")]
    Read(
        #[source]
        #[from]
        io::Error,
    ),
    #[error("Failed decoding script")]
    Decode(
        #[from]
        #[source]
        serde_json::Error,
    ),
    #[error("Unknown key {:?}", .0)]
    UnknownKey(char),
}
