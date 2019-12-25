use crate::{error::GameResult, input::Key, orient::Coord2D};
use lazy_static::lazy_static;
use std::{any::type_name, fmt, future::Future, pin::Pin, sync::Arc};
use tokio::{sync::Mutex, task};

lazy_static! {
    /// Global shared structure by all handles.
    static ref SHARED: Arc<Shared> = Arc::new(Shared {
        on_key: Mutex::new(EventHandler::default()),
        on_resize: Mutex::new(EventHandler::default()),
    });
}

/// A handle to the terminal.
#[derive(Debug)]
pub struct Handle {
    /// Shared between handles.
    shared: Arc<Shared>,
    /// Buffer of local modifications that need to be flushed.
    buf: Vec<u8>,
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Self { shared: self.shared.clone(), buf: Vec::new() }
    }
}

impl Handle {
    /// Initializes the handle to the terminal.
    pub async fn new() -> GameResult<Self> {
        let this = Self { shared: SHARED.clone(), buf: Vec::new() };

        Ok(this)
    }
}

/// A generic event handler.
struct EventHandler<E> {
    /// Function called by the handler to handle the event.
    function:
        Box<dyn FnMut(E) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>,
}

impl<E> Default for EventHandler<E> {
    fn default() -> Self {
        Self::new(|_| async {})
    }
}

impl<E> EventHandler<E> {
    /// Initializes the event handler.
    fn new<F, A>(mut function: F) -> Self
    where
        F: FnMut(E) -> A + Send + 'static,
        A: Future<Output = ()> + Send + 'static,
    {
        Self { function: Box::new(move |evt| Box::pin(function(evt))) }
    }
}

impl<T> fmt::Debug for EventHandler<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "EventHandler<{}> {{ function: {:?} }}",
            type_name::<T>(),
            &(&*self.function as *const _)
        )
    }
}

/// Shared structure by the handles.
#[derive(Debug)]
struct Shared {
    /// Event handler for key pressing.
    on_key: Mutex<EventHandler<KeyEvent>>,
    /// Event handler for resizing.
    on_resize: Mutex<EventHandler<ResizeEvent>>,
}

/// An event fired by a key pressed by the user.
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    /// Key pressed by the user.
    pub key: Key,
}

/// An event fired by a resize of the screen.
#[derive(Debug, Clone, Copy)]
pub struct ResizeEvent {
    /// New dimensions of the screen.
    pub size: Coord2D,
}
