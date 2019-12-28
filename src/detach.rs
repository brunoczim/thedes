use lazy_static::lazy_static;
use std::future::Future;
use tokio::{
    sync::{mpsc, Mutex},
    task,
};

#[derive(Debug)]
/// A message sent between detached futures and a waiter.
enum Message {
    /// Entering a detached future.
    Entering,
    /// Leaving a detached future.
    Leaving,
}

#[derive(Debug)]
/// Data of a waiter.
struct Waiter {
    /// Channel of messages from detached futures.
    receiver: mpsc::UnboundedReceiver<Message>,
    /// Count of detached threads running.
    count: u64,
}

#[derive(Debug)]
/// Global structure of detaching.
struct Detached {
    /// Waiter data.
    waiter: Mutex<Waiter>,
    /// Sender channel of detached futures.
    sender: mpsc::UnboundedSender<Message>,
}

lazy_static! {
    static ref DETACHED: Detached = {
        let (sender, receiver) = mpsc::unbounded_channel();
        Detached { waiter: Mutex::new(Waiter { receiver, count: 0 }), sender }
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
    DETACHED
        .sender
        .send(Message::Entering)
        .expect("Impossible that the receiver disconnected");

    task::spawn(async move {
        let guard = DetachGuard;
        let res = future.await;
        drop(guard);
        res
    })
}

impl Drop for DetachGuard {
    fn drop(&mut self) {
        DETACHED
            .sender
            .send(Message::Leaving)
            .expect("Impossible that the receiver disconnected (it is static)");
    }
}

/// Waits for all detached threads. Should be called by a main future as the
/// last thing;.
pub async fn wait() {
    let mut waiter = DETACHED.waiter.lock().await;

    while waiter.count > 0 {
        let msg =
            waiter.receiver.recv().await.expect(
                "Impossible that the sender disconnected (it is static)",
            );
        match msg {
            Message::Entering => {
                waiter.count =
                    waiter.count.checked_add(1).expect("Too much detaching")
            },
            Message::Leaving => {
                waiter.count =
                    waiter.count.checked_sub(1).expect("Inconsistent guard")
            },
        }
    }
}
