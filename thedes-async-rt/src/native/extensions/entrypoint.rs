use std::fmt;

use tokio::io;

#[derive(Debug)]
pub struct RunError {
    inner: io::Error,
}

impl RunError {
    fn wrap(inner: io::Error) -> Self {
        Self { inner }
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for RunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

pub fn run<F>(future: F) -> Result<F::Output, RunError>
where
    F: Future,
{
    tokio::runtime::Builder::new_multi_thread()
        .build()
        .map_err(RunError::wrap)
        .map(|runtime| runtime.block_on(future))
}
