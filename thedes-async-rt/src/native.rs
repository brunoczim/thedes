use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::PanicPayload;

pub mod extensions;

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    JoinHandle::wrap(tokio::task::spawn(future))
}

pub fn spawn_local<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    JoinHandle::wrap(tokio::task::spawn_local(future))
}

#[derive(Debug)]
pub struct JoinError {
    inner: tokio::task::JoinError,
}

impl JoinError {
    fn wrap(inner: tokio::task::JoinError) -> Self {
        Self { inner }
    }

    pub fn is_panic(&self) -> bool {
        self.inner.is_panic()
    }

    pub fn into_panic(self) -> PanicPayload {
        self.inner.into_panic()
    }
}

impl fmt::Display for JoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for JoinError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

pin_project! {
    #[derive(Debug)]
    pub struct JoinHandle<T> {
        #[pin]
        inner: tokio::task::JoinHandle<T>,
    }
}

impl<T> JoinHandle<T> {
    pub fn wrap(inner: tokio::task::JoinHandle<T>) -> Self {
        Self { inner }
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx).map_err(JoinError::wrap)
    }
}
