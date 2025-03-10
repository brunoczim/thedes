use std::{
    cell::{Cell, RefCell, RefMut},
    collections::BTreeMap,
    fmt,
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll, Waker},
};

type Token = usize;

#[derive(Debug, Clone)]
struct Queue {
    owner: Option<Token>,
    on_hold: BTreeMap<Token, Waker>,
}

impl Default for Queue {
    fn default() -> Self {
        Self::new()
    }
}

impl Queue {
    const fn new() -> Self {
        Self { owner: None, on_hold: BTreeMap::new() }
    }

    fn new_token(&self) -> Token {
        self.on_hold
            .last_key_value()
            .map(|(token, _)| *token)
            .max(self.owner)
            .map_or(0, |token| token + 1)
    }

    fn acquire(&mut self, waker: Waker, token: Token) {
        if self.owner.is_some() {
            self.on_hold.insert(token, waker);
        } else {
            self.owner = Some(token);
            waker.wake();
        }
    }

    fn try_acquire(&mut self) -> Option<Token> {
        if self.owner.is_some() {
            None
        } else {
            let token = self.new_token();
            self.owner = Some(token);
            Some(token)
        }
    }

    fn release(&mut self) {
        self.owner = None;
        if let Some((token, waker)) = self.on_hold.pop_first() {
            self.owner = Some(token);
            waker.wake();
        }
    }

    fn cancel(&mut self, token: Token) {
        if self.owner == Some(token) {
            self.release();
        } else {
            self.on_hold.remove(&token);
        }
    }
}

pub struct Mutex<T> {
    data: RefCell<T>,
    queue: Cell<Queue>,
}

impl<T> Mutex<T> {
    fn with_queue<F, A>(&self, visitor: F) -> A
    where
        F: FnOnce(&mut Queue) -> A,
    {
        let mut queue = self.queue.take();
        let output = visitor(&mut queue);
        self.queue.set(queue);
        output
    }

    pub const fn new(data: T) -> Self {
        Self { data: RefCell::new(data), queue: Cell::new(Queue::new()) }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        self.with_queue(|queue| {
            if queue.try_acquire().is_some() {
                Some(self.do_lock())
            } else {
                None
            }
        })
    }

    pub async fn lock(&self) -> MutexGuard<T> {
        let subscriber =
            Subscriber { mutex: self, state: SubscriberState::NotSubscribed };
        subscriber.await;
        self.do_lock()
    }

    fn do_lock(&self) -> MutexGuard<T> {
        MutexGuard { mutex: self, ref_mut: self.data.borrow_mut() }
    }
}

impl<T> Default for Mutex<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> fmt::Debug for Mutex<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        self.with_queue(|queue| {
            fmtr.debug_struct("Mutex")
                .field("data", &self.data)
                .field("queue", &queue)
                .finish()
        })
    }
}

#[derive(Debug)]
pub struct MutexGuard<'mutex, T> {
    mutex: &'mutex Mutex<T>,
    ref_mut: RefMut<'mutex, T>,
}

impl<'mutex, T> Deref for MutexGuard<'mutex, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.ref_mut
    }
}

impl<'mutex, T> DerefMut for MutexGuard<'mutex, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.ref_mut
    }
}

impl<'mutex, T> Drop for MutexGuard<'mutex, T> {
    fn drop(&mut self) {
        self.mutex.with_queue(|queue| queue.release());
    }
}

#[derive(Debug, Clone, Copy)]
enum SubscriberState {
    NotSubscribed,
    Subscribed(Token),
    Acquired,
}

#[derive(Debug)]
struct Subscriber<'mutex, T> {
    mutex: &'mutex Mutex<T>,
    state: SubscriberState,
}

impl<'mutex, T> Future for Subscriber<'mutex, T> {
    type Output = ();

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match self.state {
            SubscriberState::Acquired => Poll::Ready(()),
            SubscriberState::Subscribed(token) => {
                self.mutex.with_queue(|queue| {
                    if queue.owner == Some(token) {
                        self.state = SubscriberState::Acquired;
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
            },
            SubscriberState::NotSubscribed => self.mutex.with_queue(|queue| {
                let token = queue.new_token();
                queue.acquire(cx.waker().clone(), token);
                self.state = SubscriberState::Subscribed(token);
                Poll::Pending
            }),
        }
    }
}

impl<'mutex, T> Drop for Subscriber<'mutex, T> {
    fn drop(&mut self) {
        if let SubscriberState::Subscribed(token) = self.state {
            self.mutex.with_queue(|queue| {
                queue.cancel(token);
            })
        }
    }
}
