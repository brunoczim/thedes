use std::{cell::Cell, fmt, rc::Rc, task};

use thiserror::Error;

#[derive(Debug, Error)]
#[error("callback cancelled")]
pub struct Cancelled;

pub fn channel<T>() -> (Notifier<T>, Listener<T>) {
    let channel = Channel::init_connected();
    (Notifier::new(channel.clone()), Listener::new(channel))
}

struct ChannelInner<T> {
    connected: Cell<bool>,
    waker: Cell<Option<task::Waker>>,
    data: Cell<Option<T>>,
}

impl<T> fmt::Debug for ChannelInner<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        let waker = self.waker.take();
        let data = self.data.take();
        let result = fmtr
            .debug_struct("callback::Channel")
            .field("connected", &self.connected)
            .field("waker", &waker)
            .field("data", &data)
            .finish();
        self.waker.set(waker);
        self.data.set(data);
        result
    }
}

impl<T> ChannelInner<T> {
    fn init_connected() -> Self {
        Self {
            connected: Cell::new(true),
            waker: Cell::new(None),
            data: Cell::new(None),
        }
    }
}

#[derive(Debug)]
struct Channel<T> {
    inner: Rc<ChannelInner<T>>,
}

impl<T> Channel<T> {
    fn init_connected() -> Self {
        Self { inner: Rc::new(ChannelInner::init_connected()) }
    }

    fn is_connected(&self) -> bool {
        self.inner.connected.get()
    }

    fn disconnect(&self) -> bool {
        self.inner.connected.replace(false)
    }
}

impl<T> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

#[derive(Debug)]
pub struct Notifier<T> {
    channel: Channel<T>,
}

impl<T> Notifier<T> {
    fn new(channel: Channel<T>) -> Self {
        Self { channel }
    }

    pub fn send(&self, data: T) {
        self.channel.inner.data.set(Some(data));
        self.notify();
    }

    fn notify(&self) {
        if let Some(waker) = self.channel.inner.waker.take() {
            waker.wake();
        }
    }
}

impl<T> Clone for Notifier<T> {
    fn clone(&self) -> Self {
        Self { channel: self.channel.clone() }
    }
}

impl<T> Drop for Notifier<T> {
    fn drop(&mut self) {
        if Rc::strong_count(&self.channel.inner) <= 2 {
            self.channel.disconnect();
            self.notify();
        }
    }
}

#[derive(Debug)]
pub struct Listener<T> {
    channel: Channel<T>,
}

impl<T> Listener<T> {
    fn new(channel: Channel<T>) -> Self {
        Self { channel }
    }

    pub fn receive(&self) -> Option<Result<T, Cancelled>> {
        match self.channel.inner.data.take() {
            Some(data) => Some(Ok(data)),
            None if self.channel.is_connected() => None,
            None => Some(Err(Cancelled)),
        }
    }

    pub fn subscribe(&self, waker: &task::Waker) {
        let mut stored = self.channel.inner.waker.take();
        if stored.is_none() {
            stored = Some(waker.clone());
        }
        self.channel.inner.waker.set(stored);
    }
}

impl<T> Drop for Listener<T> {
    fn drop(&mut self) {
        self.channel.disconnect();
    }
}
