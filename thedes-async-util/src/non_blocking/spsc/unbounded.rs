use std::{
    cell::{Cell, UnsafeCell},
    fmt,
    mem::MaybeUninit,
    ops::Deref,
    ptr::{NonNull, null_mut},
    sync::atomic::{AtomicBool, AtomicPtr, Ordering::*},
};

use thiserror::Error;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
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

#[derive(Debug)]
#[repr(C)]
struct Node<T> {
    next: AtomicPtr<Self>,
    data: UnsafeCell<MaybeUninit<T>>,
}

struct Shared<T> {
    connected: AtomicBool,
    front: Cell<NonNull<Node<T>>>,
    back: AtomicPtr<Node<T>>,
}

impl<T> Shared<T> {
    pub fn new_connected() -> Self {
        let dummy = Box::new(Node {
            data: UnsafeCell::new(MaybeUninit::uninit()),
            next: AtomicPtr::new(null_mut()),
        });

        let dummy_non_null =
            unsafe { NonNull::new_unchecked(Box::into_raw(dummy)) };

        let this = Self {
            front: Cell::new(dummy_non_null),
            back: AtomicPtr::new(dummy_non_null.as_ptr()),
            connected: AtomicBool::new(true),
        };
        this
    }

    pub fn is_connected_weak(&self) -> bool {
        self.connected.load(Relaxed)
    }

    pub unsafe fn send(&self, message: T) -> Result<(), SendError<T>> {
        if self.connected.load(Acquire) {
            let new_node = Box::new(Node {
                data: UnsafeCell::new(MaybeUninit::new(message)),
                next: AtomicPtr::new(null_mut()),
            });
            unsafe {
                let node_non_null =
                    NonNull::new_unchecked(Box::into_raw(new_node));
                let back = self.back.load(Relaxed);
                (&mut *back).next.store(node_non_null.as_ptr(), Release);
                self.back.store(node_non_null.as_ptr(), Release);
            }
            Ok(())
        } else {
            Err(SendError::new(message))
        }
    }

    pub unsafe fn recv_one(&self) -> Result<Option<T>, RecvError> {
        unsafe {
            let next_ptr = self.front.get().as_ref().next.load(Acquire);
            match NonNull::new(next_ptr) {
                Some(next_non_null) => {
                    let data =
                        (&*next_non_null.as_ref().data.get()).as_ptr().read();
                    let _ = Box::from_raw(self.front.get().as_ptr());
                    self.front.set(next_non_null);
                    Ok(Some(data))
                },
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

    pub unsafe fn recv_many<'a>(
        &'a self,
    ) -> Result<RecvMany<'a, T>, RecvError> {
        let back = unsafe { NonNull::new_unchecked(self.back.load(Acquire)) };
        if back == self.front.get() && !self.connected.load(Acquire) {
            Err(RecvError::new())?
        }
        Ok(RecvMany { shared: self, back_limit: back })
    }
}

impl<T> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shared")
            .field("front", &self.front)
            .field("back", &self.back)
            .field("connected", &self.connected)
            .finish()
    }
}

impl<T> Drop for Shared<T> {
    fn drop(&mut self) {
        unsafe { while let Ok(Some(_)) = self.recv_one() {} }
        unsafe {
            let _ = Box::from_raw(self.front.get().as_ptr());
        }
    }
}

struct SharedPtr<T> {
    inner: NonNull<Shared<T>>,
}

impl<T> SharedPtr<T> {
    pub fn new() -> (Self, Self) {
        let shared = Shared::new_connected();
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
    back_limit: NonNull<Node<T>>,
}

unsafe impl<'a, T> Send for RecvMany<'a, T> where T: Send {}
unsafe impl<'a, T> Sync for RecvMany<'a, T> where T: Send {}

impl<'a, T> Iterator for RecvMany<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.shared.front.get() == self.back_limit {
                None?
            }
            self.shared.recv_one().ok().flatten()
        }
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
    use super::channel;

    #[test]
    fn recv_empty() {
        let (_sender, mut receiver) = channel::<u64>();
        assert_eq!(receiver.recv_one().unwrap(), None);
        assert_eq!(receiver.recv_one().unwrap(), None);
    }

    #[test]
    fn recv_empty_disconnected() {
        let (sender, mut receiver) = channel::<u64>();
        assert_eq!(receiver.recv_one().unwrap(), None);
        drop(sender);
        assert!(receiver.recv_one().is_err());
    }

    #[test]
    fn send_recv_once() {
        let (mut sender, mut receiver) = channel::<u64>();
        sender.send(93).unwrap();
        assert_eq!(receiver.recv_one().unwrap(), Some(93));
        assert_eq!(receiver.recv_one().unwrap(), None);
    }

    #[test]
    fn send_recv_twice_interleaved() {
        let (mut sender, mut receiver) = channel::<u64>();
        sender.send(4).unwrap();
        assert_eq!(receiver.recv_one().unwrap(), Some(4));
        sender.send(234).unwrap();
        assert_eq!(receiver.recv_one().unwrap(), Some(234));
        assert_eq!(receiver.recv_one().unwrap(), None);
    }

    #[test]
    fn send_recv_twice_consecutive() {
        let (mut sender, mut receiver) = channel::<u64>();
        sender.send(4).unwrap();
        sender.send(234).unwrap();
        assert_eq!(receiver.recv_one().unwrap(), Some(4));
        assert_eq!(receiver.recv_one().unwrap(), Some(234));
        assert_eq!(receiver.recv_one().unwrap(), None);
    }

    #[test]
    fn send_recv_dropped() {
        let (mut sender, mut receiver) = channel::<u64>();
        sender.send(9452).unwrap();
        drop(sender);
        assert_eq!(receiver.recv_one().unwrap(), Some(9452));
        assert!(receiver.recv_one().is_err());
    }

    #[test]
    fn send_recv_twice_dropped() {
        let (mut sender, mut receiver) = channel::<u64>();
        sender.send(9452).unwrap();
        sender.send(12).unwrap();
        drop(sender);
        assert_eq!(receiver.recv_one().unwrap(), Some(9452));
        assert_eq!(receiver.recv_one().unwrap(), Some(12));
        assert!(receiver.recv_one().is_err());
    }

    #[test]
    fn send_recv_twice_dropped_interleaved() {
        let (mut sender, mut receiver) = channel::<u64>();
        sender.send(9452).unwrap();
        assert_eq!(receiver.recv_one().unwrap(), Some(9452));
        sender.send(12).unwrap();
        drop(sender);
        assert_eq!(receiver.recv_one().unwrap(), Some(12));
        assert!(receiver.recv_one().is_err());
    }

    #[test]
    fn recv_many() {
        let (mut sender, mut receiver) = channel::<u64>();
        sender.send(1).unwrap();
        sender.send(2).unwrap();
        sender.send(3).unwrap();
        sender.send(4).unwrap();
        let recv_many = receiver.recv_many().unwrap();
        sender.send(5).unwrap();
        sender.send(6).unwrap();
        let numbers: Vec<_> = recv_many.collect();
        assert_eq!(numbers, vec![1, 2, 3, 4]);
    }

    #[test]
    fn recv_many_dropped() {
        let (mut sender, mut receiver) = channel::<u64>();
        sender.send(1).unwrap();
        assert_eq!(receiver.recv_one().unwrap(), Some(1));
        drop(sender);
        receiver.recv_many().unwrap_err();
    }

    #[test]
    fn recv_many_dropped_with_messages() {
        let (mut sender, mut receiver) = channel::<u64>();
        sender.send(1).unwrap();
        assert_eq!(receiver.recv_one().unwrap(), Some(1));
        sender.send(2).unwrap();
        sender.send(3).unwrap();
        sender.send(4).unwrap();
        drop(sender);
        let recv_many = receiver.recv_many().unwrap();
        let numbers: Vec<_> = recv_many.collect();
        assert_eq!(numbers, vec![2, 3, 4]);
    }
}
