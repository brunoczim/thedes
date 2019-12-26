use crate::{error::GameResult, input::Key, orient::Coord2D};
use crossterm::{cursor, event, event::Event, terminal, Command as _};
use std::{
    any::type_name,
    fmt::{self, Write},
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering::*},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{self, AsyncWriteExt},
    sync::{mpsc, Mutex},
    task,
};

fn event_poller(
    shared: Arc<Shared>,
    sender: mpsc::UnboundedSender<Event>,
) -> GameResult<()> {
    loop {
        if shared.dropped.load(Acquire) {
            break GameResult::Ok(());
        }

        if event::poll(Duration::from_millis(10))? {
            sender.send(event::read()?).expect("Sending message failed");
        }
    }
}

async fn event_listener(
    shared: Arc<Shared>,
    mut receiver: mpsc::UnboundedReceiver<Event>,
) -> GameResult<()> {
    while let Some(evt) = receiver.recv().await {
        match evt {
            Event::Key(evt) => {},
            Event::Resize(width, height) => {
                let evt = ResizeEvent { size: Coord2D { x: width, y: height } };
                let mut lock = shared.on_resize.lock().await;
                lock.size = evt.size;
                task::spawn((&mut lock.handler.function)(evt));
            },
            _ => (),
        }
    }

    Ok(())
}

/// A handle to the terminal.
#[derive(Debug)]
pub struct Handle {
    /// Shared between handles.
    shared: Arc<Shared>,
    /// Buffer of local modifications that need to be flushed.
    buf: String,
    /// Whether manual drop was called or not.
    dropped: bool,
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Self { shared: self.shared.clone(), buf: String::new(), dropped: false }
    }
}

impl Handle {
    /// Initializes the handle to the terminal.
    pub async fn new() -> GameResult<Self> {
        let fut = task::spawn_blocking(|| terminal::size());
        let (width, height) = fut.await??;
        let size = Coord2D { x: width, y: height };

        let on_resize = OnResize { handler: EventHandler::default(), size };
        let shared = Arc::new(Shared {
            on_key: Mutex::new(EventHandler::default()),
            on_resize: Mutex::new(on_resize),
            dropped: AtomicBool::new(false),
        });

        let mut this = Self { shared, buf: String::new(), dropped: false };

        this.save_screen()?;
        write!(this, "{}", cursor::Hide)?;
        this.flush().await?;

        let shared = this.shared.clone();
        let (sender, receiver) = mpsc::unbounded_channel();
        task::spawn_blocking(move || event_poller(shared, sender));
        let shared = this.shared.clone();
        task::spawn(async move { event_listener(shared, receiver) });

        Ok(this)
    }

    /// Flushes any temporary data on the buffer.
    pub async fn flush(&mut self) -> GameResult<()> {
        let mut stdout = io::stdout();
        stdout.write_all(self.buf.as_bytes()).await?;
        stdout.flush().await?;
        Ok(())
    }

    /// The current size of the screen.
    pub async fn size(&self) -> Coord2D {
        self.shared.on_resize.lock().await.size
    }

    /// Manual, async drop of the handle.
    pub async fn async_drop(mut self) -> GameResult<()> {
        self.dropped = true;
        if Arc::strong_count(&self.shared) == 3 {
            self.restore_screen()?;
            self.shared.dropped.store(true, Release);
        }
        self.flush().await?;

        Ok(())
    }

    /// Saves the screen previous the application.
    #[cfg(windows)]
    fn save_screen(&mut self) -> GameResult<()> {
        if terminal::EnterAlternateScreen.is_ansi_code_supported() {
            write!(self, "{}", terminal::EnterAlternateScreen.ansi_code())?;
        }
        Ok(())
    }

    /// Saves the screen previous the application.
    #[cfg(unix)]
    fn save_screen(&mut self) -> GameResult<()> {
        write!(self, "{}", terminal::EnterAlternateScreen.ansi_code())?;
        Ok(())
    }

    /// Restores the screen previous the application.
    #[cfg(windows)]
    fn restore_screen(&mut self) -> GameResult<()> {
        if terminal::EnterAlternateScreen.is_ansi_code_supported() {
            write!(self, "{}", terminal::LeaveAlternateScreen.ansi_code())?;
        }
        Ok(())
    }

    /// Restores the screen previous the application.
    #[cfg(unix)]
    fn restore_screen(&mut self) -> GameResult<()> {
        write!(self, "{}", terminal::LeaveAlternateScreen.ansi_code())?;
        Ok(())
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.dropped {
            panic!("Handle must be manually dropped by Handle::drop");
        }
    }
}

impl Write for Handle {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.buf.push_str(string);
        Ok(())
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
    /// Shared resize handler.
    on_resize: Mutex<OnResize>,
    /// Whether this shared handle has been dropped.
    dropped: AtomicBool,
}

/// Keeps track of size and the resize event handler.
#[derive(Debug)]
struct OnResize {
    /// Event handler for resizing and size of screen.
    handler: EventHandler<ResizeEvent>,
    /// Current size of the screen.
    size: Coord2D,
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
