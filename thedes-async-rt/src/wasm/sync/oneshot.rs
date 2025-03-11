use std::{
    cell::Cell,
    fmt,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use thiserror::Error;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Shared::new();
    let sender = Sender::new(shared.clone());
    let receiver = Receiver::new(shared);
    (sender, receiver)
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

#[derive(Debug, Error)]
#[error("sender disconnected")]
pub struct RecvError {
    _private: (),
}

impl RecvError {
    fn new() -> Self {
        Self { _private: () }
    }
}

#[derive(Debug)]
struct Queue<T> {
    connected: bool,
    message: Option<T>,
    recv_on_hold: Option<Waker>,
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self { connected: false, message: None, recv_on_hold: None }
    }
}

impl<T> Queue<T> {
    fn new() -> Self {
        Self { connected: false, message: None, recv_on_hold: None }
    }

    fn disconnect(&mut self) {
        self.connected = false;
        if let Some(recv_waker) = self.recv_on_hold.take() {
            recv_waker.wake();
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn read_message(&mut self, waker: &Waker) -> Option<T> {
        let maybe_message = self.message.take();
        self.recv_on_hold =
            if maybe_message.is_some() { None } else { Some(waker.clone()) };
        maybe_message
    }

    fn write_message(&mut self, message: T) {
        self.message = Some(message);
    }
}

struct Shared<T> {
    queue: Rc<Cell<Queue<T>>>,
}

impl<T> Shared<T> {
    fn new() -> Self {
        Self { queue: Rc::new(Cell::new(Queue::new())) }
    }

    fn with_queue<F, A>(&self, visitor: F) -> A
    where
        F: FnOnce(&mut Queue<T>) -> A,
    {
        let mut queue = self.queue.take();
        let output = visitor(&mut queue);
        self.queue.set(queue);
        output
    }
}

impl<T> fmt::Debug for Shared<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.with_queue(|queue| {
            f.debug_struct("Shared").field("queue", &*queue).finish()
        })
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self { queue: self.queue.clone() }
    }
}

#[derive(Debug)]
pub struct Sender<T> {
    shared: Shared<T>,
}

#[cfg(target_family = "wasm")]
unsafe impl<T> Send for Sender<T> where T: Send {}

#[cfg(target_family = "wasm")]
unsafe impl<T> Sync for Sender<T> where T: Send {}

impl<T> Sender<T> {
    fn new(shared: Shared<T>) -> Self {
        Self { shared }
    }

    pub fn send(&self, message: T) -> Result<(), SendError<T>> {
        self.shared.with_queue(|queue| {
            if queue.is_connected() {
                queue.write_message(message);
                Ok(())
            } else {
                Err(SendError::new(message))
            }
        })
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self::new(self.shared.clone())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.shared.with_queue(|queue| queue.disconnect());
    }
}

#[derive(Debug)]
pub struct Receiver<T> {
    shared: Shared<T>,
}

#[cfg(target_family = "wasm")]
unsafe impl<T> Send for Receiver<T> where T: Send {}

#[cfg(target_family = "wasm")]
unsafe impl<T> Sync for Receiver<T> where T: Send {}

impl<T> Receiver<T> {
    fn new(shared: Shared<T>) -> Self {
        Self { shared }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared.with_queue(|queue| queue.disconnect());
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, RecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.shared.with_queue(|queue| {
            if queue.is_connected() {
                match queue.read_message(cx.waker()) {
                    Some(message) => Poll::Ready(Ok(message)),
                    None => Poll::Pending,
                }
            } else {
                Poll::Ready(Err(RecvError::new()))
            }
        })
    }
}
