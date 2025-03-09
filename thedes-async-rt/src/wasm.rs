use std::{
    fmt,
    panic::AssertUnwindSafe,
    pin::Pin,
    task::{Context, Poll},
};

use extensions::{callback, task};
use futures::FutureExt;
use pin_project_lite::pin_project;
use thiserror::Error;

use crate::PanicPayload;

pub mod extensions;

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    spawn_local(future)
}

pub fn spawn_local<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    let register = callback::once::AsyncRegister::new(|callback| {
        task::detach(callback(()))
    });
    let callback_handle =
        register.listen(|()| AssertUnwindSafe(future).catch_unwind());
    JoinHandle::wrap(callback_handle)
}

#[derive(Debug, Error)]
enum JoinErrorInner {
    #[error("task was cancelled")]
    Cancelled(#[source] callback::Cancelled),
    #[error("task panicked")]
    Panic(PanicPayload),
}

#[derive(Debug)]
pub struct JoinError {
    inner: JoinErrorInner,
}

impl JoinError {
    fn wrap(inner: JoinErrorInner) -> Self {
        Self { inner }
    }

    pub fn is_panic(&self) -> bool {
        matches!(self.inner, JoinErrorInner::Panic(_))
    }

    pub fn into_panic(self) -> PanicPayload {
        match self.inner {
            JoinErrorInner::Panic(payload) => payload,
            err => panic!("expected a panic-type join error, found: {err:#?}"),
        }
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
        inner: callback::once::Listener<Result<T, PanicPayload>>,
    }
}

impl<T> JoinHandle<T> {
    fn wrap(inner: callback::once::Listener<Result<T, PanicPayload>>) -> Self {
        Self { inner }
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx).map(|result| match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(panic)) => {
                Err(JoinError::wrap(JoinErrorInner::Panic(panic)))
            },
            Err(cancelled) => {
                Err(JoinError::wrap(JoinErrorInner::Cancelled(cancelled)))
            },
        })
    }
}
