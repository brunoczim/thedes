use std::{
    any::Any,
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

pub use async_trait::async_trait as dynamic_async_trait;
pub use futures;
pub use trait_variant::make as static_async_trait;

#[cfg(not(target_family = "wasm"))]
use native as backend;

#[cfg(target_family = "wasm")]
use wasm as backend;

#[cfg(not(target_family = "wasm"))]
mod native;

#[cfg(target_family = "wasm")]
mod wasm;

pub mod extensions;

pub type PanicPayload = Box<dyn Any + Send + 'static>;

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    JoinHandle::wrap(backend::spawn(future))
}

pub fn spawn_local<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    JoinHandle::wrap(backend::spawn_local(future))
}

#[derive(Debug)]
pub struct JoinError {
    inner: backend::JoinError,
}

impl JoinError {
    fn wrap(inner: backend::JoinError) -> Self {
        Self { inner }
    }

    pub fn is_panic(&self) -> bool {
        self.inner.is_panic()
    }

    pub fn into_panic(self) -> PanicPayload {
        self.inner.into_panic()
    }

    pub fn try_into_panic(self) -> Result<PanicPayload, Self> {
        if self.is_panic() { Ok(self.into_panic()) } else { Err(self) }
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
        inner: backend::JoinHandle<T>,
    }
}

impl<T> JoinHandle<T> {
    pub fn wrap(inner: backend::JoinHandle<T>) -> Self {
        Self { inner }
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx).map_err(JoinError::wrap)
    }
}
