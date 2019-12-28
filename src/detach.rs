use lazy_static::lazy_static;
use std::{
    future::Future,
    sync::atomic::{fence, AtomicUsize, Ordering::*},
};
use tokio::{
    sync::{mpsc, Mutex},
    task,
};

#[derive(Debug)]
/// A message sent between detached futures and a waiter.
struct Finished;

#[derive(Debug)]
/// Global structure of detaching.
struct Detached {
    /// Receiver of a waiter future from a detached future.
    receiver: Mutex<mpsc::UnboundedReceiver<Finished>>,
    /// Counts how many detached threads are running.
    count: AtomicUsize,
    /// Channel of messages from detached futures.
    /// Sender channel of detached futures.
    sender: mpsc::UnboundedSender<Finished>,
}

lazy_static! {
    static ref DETACHED: Detached = {
        let (sender, receiver) = mpsc::unbounded_channel();
        Detached {
            receiver: Mutex::new(receiver),
            count: AtomicUsize::new(0),
            sender,
        }
    };
}

#[derive(Debug)]
/// A guard that sends a message on drop to tell waiter that a detached future
/// ended.
struct DetachGuard;

/// Spawns a future allowing it to be detached as long as a main future calls
/// `detach::wait`.
pub fn spawn<F>(future: F) -> task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    let mut old = DETACHED.count.load(Acquire);
    loop {
        if old == usize::max_value() {
            panic!("too much detaching");
        }

        match DETACHED.count.compare_exchange(old, old + 1, Release, Relaxed) {
            Ok(_) => break,
            Err(update) => {
                old = update;
                fence(Acquire);
            },
        }
    }

    task::spawn(async move {
        let guard = DetachGuard;
        let res = future.await;
        drop(guard);
        res
    })
}

impl Drop for DetachGuard {
    fn drop(&mut self) {
        DETACHED.sender.send(Finished).expect("Receiver is static");
    }
}

/// Waits for all detached threads. Should be called by a main future as the
/// last thing;.
pub async fn wait() {
    let mut receiver = DETACHED.receiver.lock().await;

    let mut count = DETACHED.count.load(Acquire);
    while count > 0 {
        let res =
            DETACHED.count.compare_exchange(count, count - 1, Release, Relaxed);

        match res {
            Ok(update) => {
                count = update;
                receiver.recv().await.expect("Sender is static");
            },
            Err(update) => {
                count = update;
                fence(Acquire);
            },
        }
    }
}
