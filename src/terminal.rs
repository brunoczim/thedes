use crate::{
    error::GameResult,
    input::{Event, Key, KeyEvent, ResizeEvent},
    orient::{Coord, Coord2D},
    render::{Color, TextSettings, MIN_SCREEN},
};
use crossterm::{
    cursor,
    event,
    event::Event as CrosstermEvent,
    style,
    terminal,
    Command as _,
};
use futures::future::FutureExt;
use std::{
    any::type_name,
    fmt::{self, Write},
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering::*},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{self, AsyncWriteExt},
    sync::{mpsc, Mutex},
    task,
};
use unicode_segmentation::UnicodeSegmentation;

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
        let (width, height) = task::block_in_place(|| {
            terminal::enable_raw_mode()?;
            terminal::size()
        })?;
        let bits = AtomicU32::new(width as u32 | (height as u32) << 16);

        let (key_sender, key_recv) = mpsc::unbounded_channel();
        let (resize_sender, resize_recv) = mpsc::unbounded_channel();

        let shared = Arc::new(Shared {
            key_chan: Mutex::new(key_recv),
            resize_chan: Mutex::new(resize_recv),
            stdout_lock: Mutex::new(()),
            screen_size: bits,
            dropped: AtomicBool::new(false),
        });

        let mut this = Self { shared, buf: String::new(), dropped: false };

        this.save_screen()?;
        write!(this, "{}", cursor::Hide)?;
        this.set_bg(Color::Black)?;
        this.set_fg(Color::White)?;
        this.flush().await?;

        {
            let this = this.clone();
            task::spawn(async move {
                this.event_listener(key_sender, resize_sender)
            });
        }

        this.check_screen_size(this.screen_size()).await?;

        Ok(this)
    }

    /// Manual, async drop of the handle.
    pub async fn async_drop(mut self) -> GameResult<()> {
        self.dropped = true;
        // One is us, the other is event listener.
        if Arc::strong_count(&self.shared) == 2 {
            self.restore_screen()?;
            self.shared.dropped.store(true, Release);
        }
        self.flush().await?;

        Ok(())
    }

    /// Flushes any temporary data on the buffer. Needs to be called after
    /// `write!` and `writeln!`.
    pub async fn flush(&mut self) -> GameResult<()> {
        let lock = self.shared.stdout_lock.lock().await;
        let mut stdout = io::stdout();
        stdout.write_all(self.buf.as_bytes()).await?;
        stdout.flush().await?;
        drop(lock);
        Ok(())
    }

    /// Waits for a key to be pressed.
    pub async fn listen_key(&self) -> KeyEvent {
        self.shared
            .key_chan
            .lock()
            .await
            .recv()
            .await
            .expect("Receiver cannot have disconnected")
    }

    /// Waits for the screen to be resized.
    pub async fn listen_resize(&self) -> ResizeEvent {
        self.shared
            .resize_chan
            .lock()
            .await
            .recv()
            .await
            .expect("Receiver cannot have disconnected")
    }

    /// Waits for an event to occur.
    pub async fn listen_event(&self) -> Event {
        futures::select! {
            key = self.listen_key().fuse() => Event::Key(key),
            resize = self.listen_resize().fuse() => Event::Resize(resize),
        }
    }

    /// The current size of the screen.
    pub fn screen_size(&self) -> Coord2D {
        let bits = self.shared.screen_size.load(Acquire);

        Coord2D { x: bits as u16, y: (bits >> 16) as u16 }
    }

    /// Sets the foreground color. Needs to be flushed.
    pub fn set_fg(&mut self, color: Color) -> GameResult<()> {
        let color = translate_color(color);
        write!(self, "{}", style::SetForegroundColor(color))?;
        Ok(())
    }

    /// Sets the background color. Needs to be flushed.
    pub fn set_bg(&mut self, color: Color) -> GameResult<()> {
        let color = translate_color(color);
        write!(self, "{}", style::SetBackgroundColor(color))?;
        Ok(())
    }

    /// Clears the whole screen. Needs to be flushed.
    pub fn clear_screen(&mut self) -> GameResult<()> {
        write!(self, "{}", terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    /// Goes to the given position on the screen. Needs to be flushed.
    pub fn goto(&mut self, pos: Coord2D) -> GameResult<()> {
        write!(self, "{}", cursor::MoveTo(pos.x, pos.y))?;
        Ok(())
    }

    /// Writes text with the given settings, such as left margin, right margin,
    /// and alignment based on ratio. Returns in next to the one which output
    /// stopped. Needs to be flushed.
    pub fn aligned_text(
        &mut self,
        string: &str,
        y: Coord,
        settings: TextSettings,
    ) -> GameResult<Coord> {
        let mut indices =
            string.grapheme_indices(true).map(|(i, _)| i).collect::<Vec<_>>();
        indices.push(string.len());
        let mut line: Coord = 0;
        let mut slice = &*indices;
        let screen = self.screen_size();
        let width = (screen.x - settings.lmargin - settings.rmargin) as usize;

        while slice.len() > 1 {
            let pos = if width > slice.len() {
                slice.len() - 1
            } else {
                slice[.. width]
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| i < slice.len() - 1)
                    .rfind(|&(i, &idx)| &string[idx .. slice[i + 1]] == " ")
                    .map_or(width, |(i, _)| i)
            };

            let x = (screen.x - pos as Coord) / settings.den * settings.num;
            self.goto(Coord2D { x, y: y + line })?;
            write!(self, "{}", &string[slice[0] .. slice[pos]])?;
            slice = &slice[pos ..];
            line += 1;
        }
        Ok(y + line)
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

    /// Checks if the new screen doesn't violate the min-screen size guarantee.
    async fn check_screen_size(
        &mut self,
        size: Coord2D,
    ) -> GameResult<Coord2D> {
        if size.x < MIN_SCREEN.x || size.y < MIN_SCREEN.y {
            let new_size =
                Coord2D::from_map(|axis| size[axis].min(MIN_SCREEN[axis]));
            write!(self, "{}", terminal::SetSize(new_size.x, new_size.y))?;
            let this = self.clone();
            let fut = self.flush();
            this.shared
                .screen_size
                .store(new_size.x as u32 | (new_size.y as u32) << 16, Release);
            fut.await?;

            Ok(new_size)
        } else {
            Ok(size)
        }
    }

    /// Listens for the events from the user and invoke handlers.
    async fn event_listener(
        mut self,
        key_sender: mpsc::UnboundedSender<KeyEvent>,
        resize_sender: mpsc::UnboundedSender<ResizeEvent>,
    ) -> GameResult<()> {
        loop {
            if self.shared.dropped.load(Acquire) {
                break Ok(());
            }

            let evt: GameResult<_> = task::block_in_place(|| {
                if event::poll(Duration::from_millis(10))? {
                    Ok(Some(event::read()?))
                } else {
                    Ok(None)
                }
            });

            match evt? {
                Some(CrosstermEvent::Key(key)) => {
                    if let Some(main_key) = translate_key(key.code) {
                        use event::KeyModifiers as Mod;

                        let evt = KeyEvent {
                            main_key,
                            ctrl: key.modifiers.intersects(Mod::CONTROL),
                            alt: key.modifiers.intersects(Mod::ALT),
                            shift: key.modifiers.intersects(Mod::SHIFT),
                        };

                        key_sender.send(evt).expect(
                            "Impossible that the receiver disconnected",
                        );
                    }
                },

                Some(CrosstermEvent::Resize(width, height)) => {
                    let size = self
                        .check_screen_size(Coord2D { x: width, y: height })
                        .await?;
                    self.shared
                        .screen_size
                        .store(size.x as u32 | (size.y as u32) << 16, Release);
                    let evt = ResizeEvent { size };
                    resize_sender
                        .send(evt)
                        .expect("Impossible that the receiver disconnected");
                },
                _ => (),
            }
        }
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

/// Just a helper to make code more legible. Async trait object.
type Async<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// A generic event handler.
struct EventHandler<E> {
    /// Function called by the handler to handle the event.
    function: Box<dyn FnMut(Handle, E) -> Async<GameResult<()>> + Send>,
}

impl<E> Default for EventHandler<E> {
    fn default() -> Self {
        Self::new(|_, _| async { Ok(()) })
    }
}

impl<E> EventHandler<E> {
    /// Initializes the event handler.
    fn new<F, A>(mut function: F) -> Self
    where
        F: FnMut(Handle, E) -> A + Send + 'static,
        A: Future<Output = GameResult<()>> + Send + 'static,
    {
        Self {
            function: Box::new(move |handle, evt| {
                Box::pin(function(handle, evt))
            }),
        }
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
    /// Channel to listen on key pressing.
    key_chan: Mutex<mpsc::UnboundedReceiver<KeyEvent>>,
    /// Channel to listen on resizing.
    resize_chan: Mutex<mpsc::UnboundedReceiver<ResizeEvent>>,
    /// Locks stdout.
    stdout_lock: Mutex<()>,
    /// Whether this shared handle has been dropped.
    dropped: AtomicBool,
    /// Current screen size.
    screen_size: AtomicU32,
}

/// Translates between keys of crossterm and keys of thedes.
fn translate_key(crossterm: event::KeyCode) -> Option<Key> {
    match crossterm {
        event::KeyCode::Esc => Some(Key::Esc),
        event::KeyCode::Backspace => Some(Key::Backspace),
        event::KeyCode::Enter => Some(Key::Enter),
        event::KeyCode::Up => Some(Key::Up),
        event::KeyCode::Down => Some(Key::Down),
        event::KeyCode::Left => Some(Key::Left),
        event::KeyCode::Right => Some(Key::Right),
        event::KeyCode::Char(ch) => Some(Key::Char(ch)),
        _ => None,
    }
}

fn translate_color(color: Color) -> style::Color {
    match color {
        Color::Black => style::Color::Black,
        Color::White => style::Color::White,
        Color::Red => style::Color::DarkRed,
        Color::Green => style::Color::DarkGreen,
        Color::Blue => style::Color::DarkBlue,
        Color::Magenta => style::Color::DarkMagenta,
        Color::Yellow => style::Color::DarkYellow,
        Color::Cyan => style::Color::DarkCyan,
        Color::DarkGrey => style::Color::DarkGrey,
        Color::LightGrey => style::Color::Grey,
        Color::LightRed => style::Color::Red,
        Color::LightGreen => style::Color::Green,
        Color::LightBlue => style::Color::Blue,
        Color::LightMagenta => style::Color::Magenta,
        Color::LightYellow => style::Color::Yellow,
        Color::LightCyan => style::Color::Cyan,
    }
}
