use crate::{
    detach,
    error::{exit_on_error, GameResult},
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
    fmt::{self, Write},
    mem,
    sync::{
        atomic::{AtomicU32, Ordering::*},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{self, AsyncWriteExt},
    sync::{watch, Mutex, MutexGuard},
    task,
};
use unicode_segmentation::UnicodeSegmentation;

/// A handle to the terminal.
#[derive(Debug)]
pub struct Handle {
    /// Shared between handles.
    shared: Arc<Shared>,
    /// Channel to listen on key pressing.
    key_chan: watch::Receiver<KeyEvent>,
    /// Channel to listen on resizing.
    resize_chan: watch::Receiver<ResizeEvent>,
    /// Buffer of local modifications that need to be flushed.
    buf: String,
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
            buf: String::new(),
            key_chan: self.key_chan.clone(),
            resize_chan: self.resize_chan.clone(),
        }
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

        let dummy = KeyEvent {
            main_key: Key::Enter,
            alt: false,
            ctrl: false,
            shift: false,
        };
        let (key_sender, mut key_recv) = watch::channel(dummy);
        let dummy = ResizeEvent { size: Coord2D { x: 0, y: 0 } };
        let (resize_sender, mut resize_recv) = watch::channel(dummy);
        key_recv.recv().await;
        resize_recv.recv().await;

        let shared =
            Arc::new(Shared { stdout_lock: Mutex::new(()), screen_size: bits });

        let mut this = Self {
            shared,
            buf: String::new(),
            key_chan: key_recv,
            resize_chan: resize_recv,
        };

        this.save_screen()?;
        write!(this, "{}", cursor::Hide)?;
        this.set_bg(Color::Black)?;
        this.set_fg(Color::White)?;
        this.flush().await?;

        let shared = this.shared.clone();
        detach::spawn(async move {
            exit_on_error(
                shared.event_listener(key_sender, resize_sender).await,
            )
        });

        Ok(this)
    }

    /// Flushes any temporary data on the buffer. Needs to be called after
    /// `write!` and `writeln!`.
    pub async fn flush(&mut self) -> GameResult<()> {
        self.shared.write_and_flush(self.buf.as_bytes()).await
    }

    /// Waits for a key to be pressed.
    pub async fn listen_key(&mut self) -> KeyEvent {
        self.key_chan.recv().await.expect("Sender cannot have disconnected")
    }

    /// Waits for the screen to be resized.
    pub async fn listen_resize(&mut self) -> ResizeEvent {
        self.resize_chan.recv().await.expect("Sender cannot have disconnected")
    }

    /// Waits for an event to occur.
    pub async fn listen_event(&mut self) -> Event {
        let key = self.key_chan.recv();
        let resize = self.resize_chan.recv();
        futures::select! {
            evt = key.fuse() => Event::Key(evt.expect("Sender is up")),
            evt = resize.fuse() => Event::Resize(evt.expect("Sender is up")),
        }
    }

    /// The current size of the screen.
    pub fn screen_size(&self) -> Coord2D {
        self.shared.screen_size()
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
            let x = x + settings.lmargin - settings.rmargin;
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
        if terminal::LeaveAlternateScreen.is_ansi_code_supported() {
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

impl Write for Handle {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.buf.push_str(string);
        Ok(())
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        let res = (|| {
            // One is us, the other is event listener.
            if Arc::strong_count(&self.shared) == 2 {
                task::block_in_place(|| terminal::disable_raw_mode())?;
                write!(self, "{}", cursor::Show)?;
                write!(
                    self,
                    "{}",
                    style::SetBackgroundColor(style::Color::Reset)
                )?;
                write!(
                    self,
                    "{}",
                    style::SetForegroundColor(style::Color::Reset)
                )?;
                let _ = self.restore_screen()?;
            }

            let buf = mem::replace(&mut self.buf, String::new());
            if buf.len() > 0 {
                let shared = self.shared.clone();
                detach::spawn(async move {
                    exit_on_error(shared.write_and_flush(buf.as_bytes()).await);
                });
            }
            Ok(())
        })();

        exit_on_error(res);
    }
}

/// Shared structure by the handles.
#[derive(Debug)]
struct Shared {
    /// Locks stdout.
    stdout_lock: Mutex<()>,
    /// Current screen size.
    screen_size: AtomicU32,
}

impl Shared {
    /// Listens for the events from the user and invoke handlers.
    async fn event_listener(
        &self,
        mut key_sender: watch::Sender<KeyEvent>,
        mut resize_sender: watch::Sender<ResizeEvent>,
    ) -> GameResult<()> {
        let mut stdout_lock =
            self.check_screen_size(self.screen_size(), None).await?;

        loop {
            let fut = async {
                task::block_in_place(|| {
                    if event::poll(Duration::from_millis(10))? {
                        Ok(Some(event::read()?))
                    } else {
                        Ok(None)
                    }
                })
            };
            let evt: GameResult<_> = futures::select! {
                _ = key_sender.closed().fuse() => break Ok(()),
                _ = resize_sender.closed().fuse() => break Ok(()),
                evt = fut.fuse() => evt,
            };

            match evt? {
                Some(CrosstermEvent::Key(key)) => {
                    let maybe_key = translate_key(key.code)
                        .filter(|_| stdout_lock.is_none());
                    if let Some(main_key) = maybe_key {
                        use event::KeyModifiers as Mod;

                        let evt = KeyEvent {
                            main_key,
                            ctrl: key.modifiers.intersects(Mod::CONTROL),
                            alt: key.modifiers.intersects(Mod::ALT),
                            shift: key.modifiers.intersects(Mod::SHIFT),
                        };

                        let _ = key_sender.broadcast(evt);
                    }
                },

                Some(CrosstermEvent::Resize(width, height)) => {
                    let size = Coord2D { x: width, y: height };
                    let fut = self.check_screen_size(size, stdout_lock);
                    stdout_lock = fut.await?;
                    if stdout_lock.is_none() {
                        self.screen_size.store(
                            size.x as u32 | (size.y as u32) << 16,
                            Release,
                        );
                        let evt = ResizeEvent { size };
                        let _ = resize_sender.broadcast(evt);
                    }
                },
                _ => (),
            }
        }
    }

    /// Checks if the new screen doesn't violate the min-screen size guarantee.
    async fn check_screen_size<'guard>(
        &'guard self,
        size: Coord2D,
        mut guard: Option<MutexGuard<'guard, ()>>,
    ) -> GameResult<Option<MutexGuard<'guard, ()>>> {
        if size.x < MIN_SCREEN.x || size.y < MIN_SCREEN.y {
            if guard.is_some() {
                Ok(guard)
            } else {
                guard = Some(self.stdout_lock.lock().await);
                let mut buf = String::new();
                write!(
                    buf,
                    "{}{}RESIZE {}x{}",
                    terminal::Clear(terminal::ClearType::All),
                    cursor::MoveTo(0, 0),
                    MIN_SCREEN.x,
                    MIN_SCREEN.y
                )?;
                let mut stdout = io::stdout();
                stdout.write_all(buf.as_bytes()).await?;
                stdout.flush().await?;
                Ok(guard)
            }
        } else {
            Ok(None)
        }
    }

    fn screen_size(&self) -> Coord2D {
        let bits = self.screen_size.load(Acquire);

        Coord2D { x: bits as u16, y: (bits >> 16) as u16 }
    }

    async fn write_and_flush(&self, buf: &[u8]) -> GameResult<()> {
        let lock = self.stdout_lock.lock().await;
        let mut stdout = io::stdout();
        stdout.write_all(buf).await?;
        stdout.flush().await?;
        drop(lock);
        Ok(())
    }
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
