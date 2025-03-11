use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

use crate::backend;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (sender_inner, receiver_inner) = backend::sync::oneshot::channel();
    (Sender::wrap(sender_inner), Receiver::wrap(receiver_inner))
}

#[derive(Debug)]
pub struct SendError<T> {
    inner: backend::sync::oneshot::SendError<T>,
}

impl<T> SendError<T> {
    fn wrap(inner: backend::sync::oneshot::SendError<T>) -> Self {
        Self { inner }
    }

    pub fn into_failed_message(self) -> T {
        self.inner.into_failed_message()
    }
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> std::error::Error for SendError<T>
where
    T: fmt::Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

#[derive(Debug)]
pub struct RecvError {
    inner: backend::sync::oneshot::RecvError,
}

impl RecvError {
    fn wrap(inner: backend::sync::oneshot::RecvError) -> Self {
        Self { inner }
    }
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for RecvError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

#[derive(Debug)]
pub struct Sender<T> {
    inner: backend::sync::oneshot::Sender<T>,
}

impl<T> Sender<T> {
    fn wrap(inner: backend::sync::oneshot::Sender<T>) -> Self {
        Self { inner }
    }

    pub fn send(self, message: T) -> Result<(), SendError<T>> {
        self.inner.send(message).map_err(SendError::wrap)
    }
}

#[derive(Debug)]
#[pin_project]
pub struct Receiver<T> {
    #[pin]
    inner: backend::sync::oneshot::Receiver<T>,
}

impl<T> Receiver<T> {
    fn wrap(inner: backend::sync::oneshot::Receiver<T>) -> Self {
        Self { inner }
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, RecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx).map_err(RecvError::wrap)
    }
}
