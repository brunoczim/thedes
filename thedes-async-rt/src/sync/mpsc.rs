use std::fmt;

use crate::backend;

pub fn channel<T>(buf_size: usize) -> (Sender<T>, Receiver<T>) {
    let (sender_inner, receiver_inner) = backend::sync::mpsc::channel(buf_size);
    (Sender::wrap(sender_inner), Receiver::wrap(receiver_inner))
}

#[derive(Debug)]
pub struct SendError<T> {
    inner: backend::sync::mpsc::SendError<T>,
}

impl<T> SendError<T> {
    fn wrap(inner: backend::sync::mpsc::SendError<T>) -> Self {
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
pub struct Sender<T> {
    inner: backend::sync::mpsc::Sender<T>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T> Sender<T> {
    fn wrap(inner: backend::sync::mpsc::Sender<T>) -> Self {
        Self { inner }
    }

    pub async fn send(&self, message: T) -> Result<(), SendError<T>> {
        self.inner.send(message).await.map_err(SendError::wrap)
    }
}

#[derive(Debug)]
pub struct Receiver<T> {
    inner: backend::sync::mpsc::Receiver<T>,
}

impl<T> Receiver<T> {
    fn wrap(inner: backend::sync::mpsc::Receiver<T>) -> Self {
        Self { inner }
    }

    pub async fn recv(&mut self) -> Option<T> {
        self.inner.recv().await
    }
}
