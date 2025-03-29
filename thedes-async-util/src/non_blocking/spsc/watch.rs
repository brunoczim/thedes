use std::{
    fmt,
    ops::Deref,
    ptr::{NonNull, null_mut},
    sync::atomic::{
        AtomicBool,
        AtomicPtr,
        Ordering::{self, *},
    },
};

use thiserror::Error;

pub fn channel<T>() -> (Sender<T>, Receiver<T>)
where
    T: AtomicMessage,
{
    let (sender_shared, receiver_shared) = SharedPtr::new();
    (Sender::new(sender_shared), Receiver::new(receiver_shared))
}

#[derive(Debug, Clone, Error)]
#[error("Receiver disconnected")]
pub struct SendError<T> {
    message: T,
}

impl<T> SendError<T> {
    fn new(message: T) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &T {
        &self.message
    }

    pub fn into_message(self) -> T {
        self.message
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

pub trait AtomicMessage: Sized {
    type Data;

    fn empty() -> Self;

    fn take(&self, ordering: Ordering) -> Option<Self::Data>;

    fn store(&self, value: Self::Data, ordering: Ordering);
}

struct Shared<M> {
    connected: AtomicBool,
    current: M,
}

impl<M> Shared<M>
where
    M: AtomicMessage,
{
    pub fn new_connected() -> Self {
        Self { current: M::empty(), connected: AtomicBool::new(true) }
    }

    pub fn is_connected_weak(&self) -> bool {
        self.connected.load(Relaxed)
    }

    pub fn send(&self, message: M::Data) -> Result<(), SendError<M::Data>> {
        if self.connected.load(Acquire) {
            self.current.store(message, Release);
            Ok(())
        } else {
            Err(SendError::new(message))
        }
    }

    pub fn recv(&self) -> Result<Option<M::Data>, RecvError> {
        match self.current.take(Acquire) {
            Some(data) => Ok(Some(data)),
            None => {
                if self.connected.load(Acquire) {
                    Ok(None)
                } else {
                    Err(RecvError::new())
                }
            },
        }
    }
}

impl<M> fmt::Debug for Shared<M>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shared")
            .field("current", &self.current)
            .field("connected", &self.connected)
            .finish()
    }
}

impl<M> Drop for Shared<M> {
    fn drop(&mut self) {}
}

struct SharedPtr<M> {
    inner: NonNull<Shared<M>>,
}

impl<M> SharedPtr<M>
where
    M: AtomicMessage,
{
    pub fn new() -> (Self, Self) {
        let shared = Shared::new_connected();
        let shared_boxed = Box::new(shared);

        let inner =
            unsafe { NonNull::new_unchecked(Box::into_raw(shared_boxed)) };

        (Self { inner }, Self { inner })
    }
}

impl<M> fmt::Debug for SharedPtr<M>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedPtr").field("inner", &**self).finish()
    }
}

impl<M> Deref for SharedPtr<M> {
    type Target = Shared<M>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl<M> Drop for SharedPtr<M> {
    fn drop(&mut self) {
        unsafe {
            let last = self
                .connected
                .compare_exchange(true, false, AcqRel, Acquire)
                .is_err();
            if last {
                let _ = Box::from_raw(self.inner.as_ptr());
            }
        }
    }
}

#[derive(Debug)]
pub struct Sender<M> {
    shared: SharedPtr<M>,
}

impl<M> Sender<M>
where
    M: AtomicMessage,
{
    fn new(shared: SharedPtr<M>) -> Self {
        Self { shared }
    }

    pub fn is_connected(&self) -> bool {
        self.shared.is_connected_weak()
    }

    pub fn send(&mut self, message: M::Data) -> Result<(), SendError<M::Data>> {
        self.shared.send(message)
    }
}

unsafe impl<M> Send for Sender<M>
where
    M: AtomicMessage + Send,
    M::Data: Send,
{
}

unsafe impl<M> Sync for Sender<M>
where
    M: AtomicMessage + Send,
    M::Data: Send,
{
}

#[derive(Debug)]
pub struct Receiver<M> {
    shared: SharedPtr<M>,
}

impl<M> Receiver<M>
where
    M: AtomicMessage,
{
    fn new(shared: SharedPtr<M>) -> Self {
        Self { shared }
    }

    pub fn is_connected(&self) -> bool {
        self.shared.is_connected_weak()
    }

    pub fn recv(&mut self) -> Result<Option<M::Data>, RecvError> {
        self.shared.recv()
    }
}

unsafe impl<M> Send for Receiver<M>
where
    M: AtomicMessage + Send,
    M::Data: Send,
{
}

unsafe impl<M> Sync for Receiver<M>
where
    M: AtomicMessage + Send,
    M::Data: Send,
{
}

#[derive(Debug)]
pub struct MessageBox<T> {
    inner: AtomicPtr<T>,
}

impl<T> AtomicMessage for MessageBox<T> {
    type Data = T;

    fn empty() -> Self {
        Self { inner: AtomicPtr::new(null_mut()) }
    }

    fn take(&self, ordering: Ordering) -> Option<Self::Data> {
        let ptr = self.inner.swap(null_mut(), ordering);
        if ptr.is_null() { None } else { unsafe { Some(*Box::from_raw(ptr)) } }
    }

    fn store(&self, value: Self::Data, ordering: Ordering) {
        let ptr = Box::into_raw(Box::new(value));
        self.inner.store(ptr, ordering);
    }
}

impl<T> Drop for MessageBox<T> {
    fn drop(&mut self) {
        unsafe {
            let ptr: *mut T = *self.inner.get_mut();
            if !ptr.is_null() {
                let _ = Box::from_raw(ptr);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{MessageBox, channel};

    #[test]
    fn recv_empty() {
        let (_sender, mut receiver) = channel::<MessageBox<u64>>();
        assert_eq!(receiver.recv().unwrap(), None);
    }

    #[test]
    fn recv_one() {
        let (mut sender, mut receiver) = channel::<MessageBox<u64>>();
        sender.send(12).unwrap();
        assert_eq!(receiver.recv().unwrap(), Some(12));
    }

    #[test]
    fn recv_one_then_none() {
        let (mut sender, mut receiver) = channel::<MessageBox<u64>>();
        sender.send(12).unwrap();
        assert_eq!(receiver.recv().unwrap(), Some(12));
        assert_eq!(receiver.recv().unwrap(), None);
    }

    #[test]
    fn recv_twice_then_none() {
        let (mut sender, mut receiver) = channel::<MessageBox<u64>>();
        sender.send(12).unwrap();
        assert_eq!(receiver.recv().unwrap(), Some(12));
        sender.send(13).unwrap();
        assert_eq!(receiver.recv().unwrap(), Some(13));
        assert_eq!(receiver.recv().unwrap(), None);
        assert_eq!(receiver.recv().unwrap(), None);
    }

    #[test]
    fn sender_disconnected() {
        let (_, mut receiver) = channel::<MessageBox<u64>>();
        receiver.recv().unwrap_err();
    }

    #[test]
    fn receiver_disconnected() {
        let (mut sender, _) = channel::<MessageBox<u64>>();
        sender.send(32).unwrap_err();
    }
}
