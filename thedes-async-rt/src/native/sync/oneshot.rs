use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;
use thiserror::Error;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (sender_inner, receiver_inner) = tokio::sync::oneshot::channel();
    (Sender::wrap(sender_inner), Receiver::wrap(receiver_inner))
}

#[derive(Debug, Error)]
#[error("receiver disconnected")]
pub struct SendError<T> {
    message: T,
}

impl<T> SendError<T> {
    fn new(message: T) -> Self {
        Self { message }
    }

    pub fn into_failed_message(self) -> T {
        self.message
    }
}

#[derive(Debug)]
pub struct RecvError {
    inner: tokio::sync::oneshot::error::RecvError,
}

impl RecvError {
    fn wrap(inner: tokio::sync::oneshot::error::RecvError) -> Self {
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
    inner: tokio::sync::oneshot::Sender<T>,
}

impl<T> Sender<T> {
    fn wrap(inner: tokio::sync::oneshot::Sender<T>) -> Self {
        Self { inner }
    }

    pub fn send(self, message: T) -> Result<(), SendError<T>> {
        self.inner.send(message).map_err(SendError::new)
    }
}

#[derive(Debug)]
#[pin_project]
pub struct Receiver<T> {
    #[pin]
    inner: tokio::sync::oneshot::Receiver<T>,
}

impl<T> Receiver<T> {
    fn wrap(inner: tokio::sync::oneshot::Receiver<T>) -> Self {
        Self { inner }
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, RecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx).map_err(RecvError::wrap)
    }
}
