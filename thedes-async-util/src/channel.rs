use std::sync::{Arc, Mutex};

use thiserror::Error;
use tokio::{sync::mpsc, time::Instant};

pub fn timed_channel<T>() -> (TimedSender<T>, TimedReceiver<T>) {
    let (sender_inner, receiver_inner) = mpsc::unbounded_channel();
    let sender = TimedSender::new(sender_inner);
    let receiver = TimedReceiver::new(receiver_inner);
    (sender, receiver)
}

#[derive(Debug, Clone, Error)]
#[error("Receiver disconnected")]
pub struct SendError<T> {
    payload: T,
}

impl<T> SendError<T> {
    fn new(message: Message<T>) -> Self {
        Self { payload: message.into_payload() }
    }

    pub fn payload(&self) -> &T {
        &self.payload
    }

    pub fn into_payload(self) -> T {
        self.payload
    }
}

#[derive(Debug, Clone, Error)]
#[error("Senders disconnected")]
pub struct RecvError {
    _private: (),
}

impl RecvError {
    fn new() -> Self {
        Self { _private: () }
    }
}

#[derive(Debug)]
pub struct TimedSender<T> {
    inner: Arc<Mutex<mpsc::UnboundedSender<Message<T>>>>,
}

impl<T> Clone for TimedSender<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T> TimedSender<T> {
    fn new(inner: mpsc::UnboundedSender<Message<T>>) -> Self {
        Self { inner: Arc::new(Mutex::new(inner)) }
    }

    pub fn send(&self, payload: T) -> Result<(), SendError<T>> {
        let result = {
            let sender_inner = self.inner.lock().expect("poisoned lock");
            let message = Message::new(payload);
            sender_inner.send(message)
        };
        result.map_err(|error| SendError::new(error.0))
    }

    pub fn send_many<I>(&self, payloads: I) -> Result<(), SendError<T>>
    where
        I: IntoIterator<Item = T>,
    {
        let mut result = Ok(());
        {
            let sender_inner = self.inner.lock().expect("poisoned lock");
            for payload in payloads {
                let message = Message::new(payload);
                result = sender_inner.send(message);
                if result.is_err() {
                    break;
                }
            }
        };
        result.map_err(|error| SendError::new(error.0))
    }
}

#[derive(Debug)]
pub struct TimedReceiver<T> {
    inner: mpsc::UnboundedReceiver<Message<T>>,
    future_message: Option<Message<T>>,
}

impl<T> TimedReceiver<T> {
    fn new(inner: mpsc::UnboundedReceiver<Message<T>>) -> Self {
        Self { inner, future_message: None }
    }

    pub async fn recv(
        &mut self,
        buf: &mut Vec<Message<T>>,
    ) -> Result<(), RecvError> {
        self.recv_until(Instant::now(), buf).await
    }

    pub async fn recv_until(
        &mut self,
        limit: Instant,
        buf: &mut Vec<Message<T>>,
    ) -> Result<(), RecvError> {
        let mut should_read_inner = true;
        if let Some(future_message) = self.future_message.take() {
            if future_message.time() <= limit {
                buf.push(future_message);
            } else {
                should_read_inner = false;
                self.future_message = Some(future_message);
            }
        }

        if should_read_inner {
            loop {
                let Some(message) = self.inner.recv().await else {
                    Err(RecvError::new())?
                };
                if message.time() <= limit {
                    buf.push(message);
                } else {
                    self.future_message = Some(message);
                    break;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Message<T> {
    time: Instant,
    payload: T,
}

impl<T> Message<T> {
    fn new(payload: T) -> Self {
        Self { time: Instant::now(), payload }
    }

    pub fn time(&self) -> Instant {
        self.time
    }

    pub fn payload(&self) -> &T {
        &self.payload
    }

    pub fn into_payload(self) -> T {
        self.payload
    }
}
