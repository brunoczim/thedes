use crate::{
    error::{exit_on_error, GameResult},
    input::{Key, KeyEvent, ResizeEvent},
    orient::{Coord, Coord2D},
    render::{Color, TextSettings},
};
use crossterm::{cursor, event, event::Event, style, terminal, Command as _};
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
    sync::Mutex,
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

        let shared = Arc::new(Shared {
            on_key: Mutex::new(EventHandler::default()),
            on_resize: Mutex::new(EventHandler::default()),
            stdout_lock: Mutex::new(()),
            screen_size: bits,
            dropped: AtomicBool::new(false),
        });

        let mut this = Self { shared, buf: String::new(), dropped: false };

        this.save_screen()?;
        write!(this, "{}", cursor::Hide)?;
        this.flush().await?;

        {
            let this = this.clone();
            task::spawn(async move { this.event_listener() });
        }

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

    /// The current size of the screen.
    pub fn screen_size(&self) -> Coord2D {
        let bits = self.shared.screen_size.load(Acquire);

        Coord2D { x: bits as u16, y: (bits >> 16) as u16 }
    }

    /// Sets the screen size to the given size and dispatches the resize event.
    pub async fn set_screen_size(&mut self, size: Coord2D) -> GameResult<()> {
        write!(self, "{}", terminal::SetSize(size.x, size.y))?;
        let this = self.clone();
        let fut = self.flush();
        this.shared
            .screen_size
            .store(size.x as u32 | (size.y as u32) << 16, Release);
        fut.await?;
        Ok(())
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

    /// Sets the given function as callback when a key is pressed.
    pub async fn on_key<F, A>(&self, function: F)
    where
        F: FnMut(Handle, KeyEvent) -> A + Send + 'static,
        A: Future<Output = GameResult<()>> + Send + 'static,
    {
        *self.shared.on_key.lock().await = EventHandler::new(function);
    }

    /// Sets the given function as callback when the screen is resized.
    pub async fn on_resize<F, A>(&self, function: F)
    where
        F: FnMut(Handle, ResizeEvent) -> A + Send + 'static,
        A: Future<Output = GameResult<()>> + Send + 'static,
    {
        *self.shared.on_resize.lock().await = EventHandler::new(function);
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

    /// Listens for the events from the user and invoke handlers.
    async fn event_listener(self) -> GameResult<()> {
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
                Some(Event::Key(key)) => {
                    if let Some(main_key) = translate_key(key.code) {
                        use event::KeyModifiers as Mod;

                        let evt = KeyEvent {
                            main_key,
                            ctrl: key.modifiers.intersects(Mod::CONTROL),
                            alt: key.modifiers.intersects(Mod::ALT),
                            shift: key.modifiers.intersects(Mod::SHIFT),
                        };

                        let this = self.clone();
                        let this_arg = this.clone();
                        let mut lock = self.shared.on_key.lock().await;
                        let fut = (&mut lock.function)(this_arg, evt);
                        task::spawn(async move { exit_on_error(fut.await) });
                    }
                },

                Some(Event::Resize(width, height)) => {
                    let size = Coord2D { x: width, y: height };
                    self.shared
                        .screen_size
                        .store(size.x as u32 | (size.y as u32) << 16, Release);
                    let evt = ResizeEvent { size };
                    let this = self.clone();
                    let this_arg = this.clone();
                    let mut lock = self.shared.on_resize.lock().await;
                    let fut = (&mut lock.function)(this_arg, evt);
                    task::spawn(async move { exit_on_error(fut.await) });
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
    /// Event handler for key pressing.
    on_key: Mutex<EventHandler<KeyEvent>>,
    /// Event handler for screen resizing.
    on_resize: Mutex<EventHandler<ResizeEvent>>,
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
