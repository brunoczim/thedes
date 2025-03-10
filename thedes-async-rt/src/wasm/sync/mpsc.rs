use std::{
    cell::Cell,
    collections::{BTreeMap, HashMap, VecDeque},
    fmt,
    mem,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use thiserror::Error;

type Token = usize;

pub fn channel<T>(buf_size: usize) -> (Sender<T>, Receiver<T>) {
    let shared = Shared::new(buf_size);
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

#[derive(Debug)]
struct SendOnHold<T> {
    waker: Waker,
    message: T,
}

#[derive(Debug)]
struct Queue<T> {
    connected: bool,
    buf_size: usize,
    messages: VecDeque<T>,
    recv_on_hold: Option<Waker>,
    sends_on_hold: BTreeMap<Token, SendOnHold<T>>,
    failed_sends: HashMap<Token, T>,
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self {
            connected: false,
            buf_size: 0,
            messages: VecDeque::new(),
            recv_on_hold: None,
            sends_on_hold: BTreeMap::new(),
            failed_sends: HashMap::new(),
        }
    }
}

impl<T> Queue<T> {
    fn new(buf_size: usize) -> Self {
        Self {
            connected: true,
            buf_size,
            messages: VecDeque::with_capacity(buf_size),
            recv_on_hold: None,
            sends_on_hold: BTreeMap::new(),
            failed_sends: HashMap::new(),
        }
    }

    fn new_token(&self) -> Token {
        self.sends_on_hold
            .last_key_value()
            .map(|(token, _)| *token)
            .map_or(0, |token| token + 1)
    }

    fn disconnect(&mut self) {
        self.connected = false;
        if let Some(recv_waker) = self.recv_on_hold.take() {
            recv_waker.wake();
            for (token, send_on_hold) in mem::take(&mut self.sends_on_hold) {
                self.failed_sends.insert(token, send_on_hold.message);
                send_on_hold.waker.wake();
            }
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn read_message(&mut self, waker: &Waker) -> Option<T> {
        let maybe_message = self.messages.pop_front();
        if maybe_message.is_some() {
            if let Some((_, send_on_hold)) = self.sends_on_hold.pop_first() {
                self.messages.push_back(send_on_hold.message);
                send_on_hold.waker.wake();
            }
            self.recv_on_hold = None;
        } else {
            self.recv_on_hold = Some(waker.clone());
        }
        maybe_message
    }

    fn write_message(&mut self, message: T, waker: &Waker) -> Option<Token> {
        if self.messages.len() > self.buf_size {
            let token = self.new_token();
            self.sends_on_hold
                .insert(token, SendOnHold { message, waker: waker.clone() });
            Some(token)
        } else {
            self.messages.push_back(message);
            None
        }
    }

    fn check_token(&mut self, token: Token) -> Result<bool, SendError<T>> {
        if let Some(failed_message) = self.failed_sends.remove(&token) {
            Err(SendError::new(failed_message))
        } else {
            Ok(self.sends_on_hold.contains_key(&token))
        }
    }

    fn cancel_token(&mut self, token: Token) {
        self.failed_sends.remove(&token);
        self.sends_on_hold.remove(&token);
    }
}

struct Shared<T> {
    queue: Rc<Cell<Queue<T>>>,
}

impl<T> Shared<T> {
    fn new(buf_size: usize) -> Self {
        Self { queue: Rc::new(Cell::new(Queue::new(buf_size))) }
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

    fn connected_count(&self) -> usize {
        Rc::strong_count(&self.queue)
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

    pub async fn send(&self, message: T) -> Result<(), SendError<T>> {
        let subscriber = SendSubscriber {
            shared: &self.shared,
            state: SendSubscriberState::Init { message },
        };
        subscriber.await
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self::new(self.shared.clone())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.shared.connected_count() <= 2 {
            self.shared.with_queue(|queue| queue.disconnect());
        }
    }
}

#[derive(Debug)]
enum SendSubscriberState<T> {
    Init { message: T },
    Waiting { token: Token },
    Done,
}

#[derive(Debug)]
struct SendSubscriber<'a, T> {
    shared: &'a Shared<T>,
    state: SendSubscriberState<T>,
}

impl<'a, T> Future for SendSubscriber<'a, T> {
    type Output = Result<(), SendError<T>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        match mem::replace(&mut this.state, SendSubscriberState::Done) {
            SendSubscriberState::Init { message } => {
                this.shared.with_queue(|queue| {
                    if queue.is_connected() {
                        match queue.write_message(message, cx.waker()) {
                            Some(token) => {
                                this.state =
                                    SendSubscriberState::Waiting { token };
                                Poll::Pending
                            },
                            None => Poll::Ready(Ok(())),
                        }
                    } else {
                        Poll::Ready(Err(SendError::new(message)))
                    }
                })
            },

            SendSubscriberState::Waiting { token } => {
                this.shared.with_queue(|queue| match queue.check_token(token) {
                    Ok(false) => {
                        this.state = SendSubscriberState::Waiting { token };
                        Poll::Pending
                    },
                    Ok(true) => Poll::Ready(Ok(())),
                    Err(failed) => Poll::Ready(Err(failed)),
                })
            },

            SendSubscriberState::Done => Poll::Ready(Ok(())),
        }
    }
}

impl<'a, T> Drop for SendSubscriber<'a, T> {
    fn drop(&mut self) {
        if let SendSubscriberState::Waiting { token } = &self.state {
            self.shared.with_queue(|queue| queue.cancel_token(*token));
        }
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

    pub async fn recv(&mut self) -> Option<T> {
        let subscriber = RecvSubscriber { shared: &self.shared };
        subscriber.await
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared.with_queue(|queue| queue.disconnect());
    }
}

#[derive(Debug)]
struct RecvSubscriber<'a, T> {
    shared: &'a Shared<T>,
}

impl<'a, T> Future for RecvSubscriber<'a, T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.shared.with_queue(|queue| {
            if queue.is_connected() {
                match queue.read_message(cx.waker()) {
                    Some(message) => Poll::Ready(Some(message)),
                    None => Poll::Pending,
                }
            } else {
                Poll::Ready(None)
            }
        })
    }
}
