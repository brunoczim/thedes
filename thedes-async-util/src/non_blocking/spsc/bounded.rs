use std::{
    cell::{Cell, UnsafeCell},
    fmt,
    mem::MaybeUninit,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering::*},
};

use thiserror::Error;

pub fn channel<T>(buf_size: usize) -> (Sender<T>, Receiver<T>) {
    if buf_size == 0 {
        panic!(
            "non-blocking SPSC bounded channel cannot have zero-sized buffer"
        );
    }
    let (sender_shared, receiver_shared) = SharedPtr::new(buf_size);
    (Sender::new(sender_shared), Receiver::new(receiver_shared))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SendErrorKind {
    #[error("Buffer is full")]
    Overflow,
    #[error("Receiver disconnected")]
    Disconnected,
}

#[derive(Debug, Clone, Error)]
#[error("Receiver disconnected")]
pub struct SendError<T> {
    #[source]
    kind: SendErrorKind,
    message: T,
}

impl<T> SendError<T> {
    fn new(kind: SendErrorKind, message: T) -> Self {
        Self { kind, message }
    }

    pub fn kind(&self) -> SendErrorKind {
        self.kind
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

#[repr(C)]
struct Shared<T> {
    connected: AtomicBool,
    unread: AtomicUsize,
    front: Cell<usize>,
    back: Cell<usize>,
    buf: Box<[UnsafeCell<MaybeUninit<T>>]>,
}

impl<T> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shared")
            .field("connected", &self.connected)
            .field("unread", &self.unread)
            .field("buf", &self.buf)
            .finish()
    }
}

impl<T> Shared<T> {
    pub fn new_connected(buf_size: usize) -> Self {
        Self {
            connected: AtomicBool::new(true),
            unread: AtomicUsize::new(0),
            front: Cell::new(0),
            back: Cell::new(0),
            buf: (0 .. buf_size)
                .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
                .collect(),
        }
    }

    pub fn is_connected_weak(&self) -> bool {
        self.connected.load(Relaxed)
    }

    pub unsafe fn send(&self, message: T) -> Result<(), SendError<T>> {
        let unread = self.unread.load(Acquire);
        let connected = self.connected.load(Acquire);
        if !connected {
            return Err(SendError::new(SendErrorKind::Disconnected, message));
        }
        if unread == self.buf.len() {
            return Err(SendError::new(SendErrorKind::Overflow, message));
        }
        unsafe {
            let back = self.back.get();
            (*self.buf[back].get()).write(message);
            self.back.set((back + 1) % self.buf.len());
            self.unread.fetch_add(1, Release);
        }
        Ok(())
    }

    pub unsafe fn recv_one(&self) -> Result<Option<T>, RecvError> {
        let unread = self.unread.load(Acquire);
        if unread == 0 {
            if self.connected.load(Acquire) {
                return Ok(None);
            }
            return Err(RecvError::new());
        }
        unsafe {
            let front = self.front.get();
            let message = (*self.buf[front].get()).as_ptr().read();
            self.front.set((front + 1) % self.buf.len());
            self.unread.fetch_sub(1, Release);
            Ok(Some(message))
        }
    }

    pub unsafe fn recv_many<'a>(
        &'a self,
    ) -> Result<RecvMany<'a, T>, RecvError> {
        let unread = self.unread.load(Acquire);
        if unread == 0 && !self.connected.load(Acquire) {
            Err(RecvError::new())?
        }
        Ok(RecvMany { shared: self, count: unread })
    }
}

impl<T> Drop for Shared<T> {
    fn drop(&mut self) {
        unsafe { while let Ok(Some(_)) = self.recv_one() {} }
    }
}

struct SharedPtr<T> {
    inner: NonNull<Shared<T>>,
}

impl<T> SharedPtr<T> {
    pub fn new(buf_size: usize) -> (Self, Self) {
        let shared = Shared::new_connected(buf_size);
        let shared_boxed = Box::new(shared);

        let inner =
            unsafe { NonNull::new_unchecked(Box::into_raw(shared_boxed)) };

        (Self { inner }, Self { inner })
    }
}

impl<T> fmt::Debug for SharedPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedPtr").field("inner", &**self).finish()
    }
}

impl<T> Deref for SharedPtr<T> {
    type Target = Shared<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl<T> Drop for SharedPtr<T> {
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
pub struct RecvMany<'a, T> {
    shared: &'a Shared<T>,
    count: usize,
}

impl<'a, T> Iterator for RecvMany<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            None?
        }
        self.count -= 1;
        unsafe { self.shared.recv_one().ok().flatten() }
    }
}

#[derive(Debug)]
pub struct Sender<T> {
    shared: SharedPtr<T>,
}

impl<T> Sender<T> {
    fn new(shared: SharedPtr<T>) -> Self {
        Self { shared }
    }

    pub fn is_connected(&self) -> bool {
        self.shared.is_connected_weak()
    }

    pub fn send(&mut self, message: T) -> Result<(), SendError<T>> {
        unsafe { self.shared.send(message) }
    }
}

unsafe impl<T> Send for Sender<T> where T: Send {}
unsafe impl<T> Sync for Sender<T> where T: Send {}

#[derive(Debug)]
pub struct Receiver<T> {
    shared: SharedPtr<T>,
}

impl<T> Receiver<T> {
    fn new(shared: SharedPtr<T>) -> Self {
        Self { shared }
    }

    pub fn is_connected(&self) -> bool {
        self.shared.is_connected_weak()
    }

    pub fn recv_one(&mut self) -> Result<Option<T>, RecvError> {
        unsafe { self.shared.recv_one() }
    }

    pub fn recv_many<'a>(&'a mut self) -> Result<RecvMany<'a, T>, RecvError> {
        unsafe { self.shared.recv_many() }
    }
}

unsafe impl<T> Send for Receiver<T> where T: Send {}
unsafe impl<T> Sync for Receiver<T> where T: Send {}

#[cfg(test)]
mod test {
    use crate::non_blocking::spsc::bounded::SendErrorKind;

    use super::channel;

    #[test]
    #[should_panic]
    fn zero_sized_buf_panics() {
        channel::<u64>(0);
    }

    #[test]
    fn recv_empty() {
        let (_sender, mut receiver) = channel::<u64>(3);
        assert_eq!(receiver.recv_one().unwrap(), None);
    }

    #[test]
    fn simple_send_recv_ok() {
        let (mut sender, mut receiver) = channel::<u64>(3);
        sender.send(1).unwrap();
        sender.send(2).unwrap();
        sender.send(3).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 1);
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 2);
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 3);
    }

    #[test]
    fn simple_send_recv_ok_interleaved() {
        let (mut sender, mut receiver) = channel::<u64>(3);
        sender.send(1).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 1);
        sender.send(2).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 2);
        sender.send(3).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 3);
    }

    #[test]
    fn send_recv_ok_interleaved_wrap_around() {
        let (mut sender, mut receiver) = channel::<u64>(3);
        sender.send(1).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 1);
        sender.send(2).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 2);
        sender.send(3).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 3);
        sender.send(4).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 4);
        sender.send(5).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 5);
    }

    #[test]
    fn send_overflow() {
        let (mut sender, mut receiver) = channel::<u64>(3);
        sender.send(1).unwrap();
        sender.send(2).unwrap();
        sender.send(3).unwrap();
        assert_eq!(sender.send(4).unwrap_err().kind(), SendErrorKind::Overflow);
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 1);
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 2);
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 3);
        assert_eq!(receiver.recv_one().unwrap(), None);
        sender.send(5).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 5);
    }

    #[test]
    fn sender_disconnected() {
        let (_, mut receiver) = channel::<u64>(3);
        receiver.recv_one().unwrap_err();
    }

    #[test]
    fn receiver_disconnected() {
        let (mut sender, _) = channel::<u64>(3);
        assert_eq!(
            sender.send(4).unwrap_err().kind(),
            SendErrorKind::Disconnected
        );
    }

    #[test]
    fn recv_many() {
        let (mut sender, mut receiver) = channel::<u64>(6);
        sender.send(1).unwrap();
        sender.send(2).unwrap();
        sender.send(3).unwrap();

        let recv_many = receiver.recv_many().unwrap();

        sender.send(4).unwrap();

        let messages: Vec<_> = recv_many.collect();

        assert_eq!(messages, vec![1, 2, 3]);
    }

    #[test]
    fn recv_many_but_sender_dropped() {
        let (mut sender, mut receiver) = channel::<u64>(3);
        sender.send(1).unwrap();
        assert_eq!(receiver.recv_one().unwrap().unwrap(), 1);
        drop(sender);
        receiver.recv_many().unwrap_err();
    }
}
