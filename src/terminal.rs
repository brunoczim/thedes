use crate::{
    error::{ErrorExt, Result},
    graphics::{Color, Color2, GString, Grapheme, Style, Tile},
    input::{translate_key, Event, KeyEvent, ResizeEvent},
    math::plane::{Coord2, Nat},
};
use crossterm::{
    cursor,
    event::{self, Event as CrosstermEvent},
    style,
    terminal,
    Command,
};
use ndarray::{Array, Ix2};
use std::{
    collections::BTreeSet,
    fmt,
    fmt::Write,
    future::Future,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering::*},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{self, AsyncWriteExt},
    sync::{watch, Mutex, MutexGuard},
    task,
    time,
};

/// A terminal configuration builder.
#[derive(Debug, Clone)]
pub struct Builder {
    min_screen: Coord2<Nat>,
    frame_rate: Duration,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Initializes this configuration builder.
    pub fn new() -> Self {
        Self {
            min_screen: Coord2 { x: 80, y: 25 },
            frame_rate: Duration::from_millis(200),
        }
    }

    /// Builderures the minimum screen size for the application.
    pub fn min_screen(self, min_screen: Coord2<Nat>) -> Self {
        Self { min_screen, ..self }
    }

    /// Builderures the rate that the screen is updated.
    pub fn frame_rate(self, frame_rate: Duration) -> Self {
        Self { frame_rate, ..self }
    }

    /// Starts the application and gives it a handle to the terminal. When the
    /// given start function finishes, the application's execution stops as
    /// well.
    pub async fn run<F, A, T>(self, start: F) -> Result<T>
    where
        F: FnOnce(Handle) -> A + Send + 'static,
        A: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let dummy = Event::Resize(ResizeEvent { size: Coord2 { x: 0, y: 0 } });
        let (sender, mut receiver) = watch::channel(dummy);
        receiver.recv().await;

        let handle = self.finish(receiver).await?;
        handle.setup().await?;

        let main_fut = {
            let handle = handle.clone();
            tokio::spawn(async move { start(handle).await })
        };

        let listener_fut = {
            let handle = handle.clone();
            tokio::spawn(async move { handle.event_listener(sender).await })
        };

        let renderer_fut = {
            let handle = handle.clone();
            tokio::spawn(async move { handle.renderer().await })
        };

        let result = tokio::select! {
            result = main_fut => result,
            result = listener_fut => result.map(|_| aux_must_fail()),
            result = renderer_fut => result.map(|_| aux_must_fail()),
        };

        let _ = handle.cleanup().await;
        // this is a double result since join can fail
        // convert join error into Box<dyn Error>
        result?
    }

    async fn finish(
        self,
        event_chan: watch::Receiver<Event>,
    ) -> Result<Handle> {
        let (width, height) = task::block_in_place(|| {
            terminal::enable_raw_mode()?;
            terminal::size()
        })?;
        let screen_size = Coord2 { x: width as Nat, y: height as Nat };
        let size_bits = AtomicU32::new(width as u32 | (height as u32) << 16);
        let shared = Arc::new(Shared {
            cleanedup: AtomicBool::new(false),
            min_screen: self.min_screen,
            event_chan: Mutex::new(event_chan),
            screen_size: size_bits,
            stdout: Mutex::new(io::stdout()),
            frame_rate: self.frame_rate,
            screen_contents: Mutex::new(ScreenContents::blank(screen_size)),
        });
        Ok(Handle { shared })
    }
}

#[inline(never)]
#[cold]
fn aux_must_fail() -> ! {
    panic!("Auxiliary task should not finish before main task unless it failed")
}

/// A handle to the terminal. It uses atomic reference counting.
#[derive(Debug, Clone)]
pub struct Handle {
    shared: Arc<Shared>,
}

impl Handle {
    /// Returns current screen size.
    pub fn screen_size(&self) -> Coord2<Nat> {
        let bits = self.shared.screen_size.load(Acquire);
        Coord2 { x: bits as Nat, y: (bits >> 16) as Nat }
    }

    /// Returns the mininum screen size.
    pub fn min_screen(&self) -> Coord2<Nat> {
        self.shared.min_screen
    }

    /// Listens for an event to happen. Waits until an event is available.
    pub async fn listen_event(&self) -> Result<Event> {
        self.shared
            .event_chan
            .lock()
            .await
            .recv()
            .await
            .ok_or_else(|| ListenerFailed.into())
    }

    /// Sets tile contents at a given position.
    pub async fn lock_screen<'handle>(&'handle self) -> Screen<'handle> {
        Screen {
            handle: self,
            contents: self.shared.screen_contents.lock().await,
        }
    }

    async fn event_listener(&self, sender: watch::Sender<Event>) -> Result<()> {
        let mut stdout = None;
        self.check_screen_size(self.screen_size(), &mut stdout).await?;

        while !self.shared.cleanedup.load(Acquire) {
            let evt = task::block_in_place(|| {
                match event::poll(Duration::from_millis(10))? {
                    true => event::read().map(Some),
                    false => Ok(None),
                }
            });

            match evt? {
                Some(CrosstermEvent::Key(key)) => {
                    let maybe_key =
                        translate_key(key.code).filter(|_| stdout.is_none());
                    if let Some(main_key) = maybe_key {
                        use event::KeyModifiers as Mod;

                        let evt = KeyEvent {
                            main_key,
                            ctrl: key.modifiers.intersects(Mod::CONTROL),
                            alt: key.modifiers.intersects(Mod::ALT),
                            shift: key.modifiers.intersects(Mod::SHIFT),
                        };

                        let _ = sender.broadcast(Event::Key(evt));
                    }
                },

                Some(CrosstermEvent::Resize(width, height)) => {
                    let size = Coord2 { x: width, y: height };
                    self.check_screen_size(size, &mut stdout).await?;
                    if stdout.is_none() {
                        let mut screen = self.lock_screen().await;
                        screen.resize(size).await?;

                        let evt = ResizeEvent { size };
                        let _ = sender.broadcast(Event::Resize(evt));
                    }
                },

                _ => (),
            }
        }

        Ok(())
    }

    async fn renderer(&self) -> Result<()> {
        let mut interval = time::interval(self.shared.frame_rate);
        let mut buf = String::new();
        while !self.shared.cleanedup.load(Acquire) {
            interval.tick().await;
            let screen_size = self.screen_size();
            let mut screen = self.lock_screen().await;
            buf.clear();

            let mut cursor = Coord2 { x: 0, y: 0 };
            let mut colors = Color2::default();
            write!(
                buf,
                "{}{}{}",
                style::SetForegroundColor(colors.fg.translate()),
                style::SetBackgroundColor(colors.bg.translate()),
                cursor::MoveTo(cursor.x as u16, cursor.y as u16),
            )?;

            for &coord in screen.contents.changed.iter() {
                if cursor != coord {
                    write!(
                        buf,
                        "{}",
                        cursor::MoveTo(coord.x as u16, coord.y as u16)
                    )?;
                }
                cursor = coord;

                let tile = screen.get(cursor);
                if colors.bg != tile.colors.bg {
                    let color = tile.colors.bg.translate();
                    write!(buf, "{}", style::SetBackgroundColor(color))?;
                }
                if colors.fg != tile.colors.fg {
                    let color = tile.colors.fg.translate();
                    write!(buf, "{}", style::SetForegroundColor(color))?;
                }
                colors = tile.colors;

                write!(buf, "{}", tile.grapheme)?;

                if cursor.x <= screen_size.x {
                    cursor.x += 1;
                }
            }

            let stdout = &mut self.shared.stdout.lock().await;
            write_and_flush(buf.as_bytes(), stdout).await?;

            screen.contents.next_tick();
        }

        Ok(())
    }

    async fn check_screen_size<'guard>(
        &'guard self,
        size: Coord2<Nat>,
        guard: &mut Option<MutexGuard<'guard, io::Stdout>>,
    ) -> Result<()> {
        if size.x < self.shared.min_screen.x
            || size.y < self.shared.min_screen.y
        {
            if guard.is_none() {
                let mut stdout = self.shared.stdout.lock().await;
                let buf = format!(
                    "{}{}RESIZE {}x{}",
                    terminal::Clear(terminal::ClearType::All),
                    cursor::MoveTo(0, 0),
                    self.shared.min_screen.x,
                    self.shared.min_screen.y,
                );
                write_and_flush(buf.as_bytes(), &mut stdout).await?;
                *guard = Some(stdout);
            }
        } else {
            *guard = None;
        }
        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        let mut buf = String::new();
        save_screen(&mut buf)?;
        write!(
            buf,
            "{}{}{}{}",
            style::SetBackgroundColor(style::Color::Black),
            style::SetForegroundColor(style::Color::White),
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
        )?;
        let mut guard = self.shared.stdout.lock().await;
        write_and_flush(buf.as_bytes(), &mut guard).await?;
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        task::block_in_place(|| terminal::disable_raw_mode())?;
        let mut buf = String::new();
        write!(buf, "{}", cursor::Show)?;
        restore_screen(&mut buf)?;
        let mut guard = self.shared.stdout.lock().await;
        write_and_flush(buf.as_bytes(), &mut guard).await?;
        self.shared.cleanedup.store(true, Release);
        Ok(())
    }
}

#[derive(Debug)]
struct ScreenContents {
    old: Array<Tile, Ix2>,
    curr: Array<Tile, Ix2>,
    changed: BTreeSet<Coord2<Nat>>,
}

impl ScreenContents {
    fn blank(dim: Coord2<Nat>) -> Self {
        let curr = Array::default(ndarray::Ix2(dim.y as usize, dim.x as usize));
        let old = curr.clone();
        Self { curr, old, changed: BTreeSet::new() }
    }

    fn clear(&mut self, dim: Coord2<Nat>) {
        self.curr =
            Array::default(ndarray::Ix2(dim.y as usize, dim.x as usize));
        self.old = self.curr.clone();
        self.changed.clear();
    }

    fn next_tick(&mut self) {
        self.changed.clear();
        let (old, curr) = (&mut self.old, &self.curr);
        old.clone_from(curr);
    }
}

/// A locked screen handle with exclusive access to it.
#[derive(Debug)]
pub struct Screen<'handle> {
    handle: &'handle Handle,
    contents: MutexGuard<'handle, ScreenContents>,
}

impl<'handle> Screen<'handle> {
    /// Returns a handle to the underlying terminal.
    pub fn handle(&self) -> &'handle Handle {
        self.handle
    }

    /// Sets the contents of a given tile. This operation is buffered.
    pub fn set(&mut self, pos: Coord2<Nat>, tile: Tile) {
        if self.contents.old[[pos.y as usize, pos.x as usize]] != tile {
            self.contents.changed.insert(pos);
        } else {
            self.contents.changed.remove(&pos);
        }
        self.contents.curr[[pos.y as usize, pos.x as usize]] = tile;
    }

    /// Gets the contents of a given tile consistently with the buffer.
    pub fn get(&self, pos: Coord2<Nat>) -> &Tile {
        &self.contents.curr[[pos.y as usize, pos.x as usize]]
    }

    /// Sets every tile into a whitespace grapheme with the given colors.
    pub fn clear(&mut self, bg: Color) {
        let size = self.handle.screen_size();
        let tile = Tile {
            colors: Color2 { bg, ..Color2::default() },
            grapheme: Grapheme::space(),
        };

        for y in 0 .. size.y {
            for x in 0 .. size.x {
                self.set(Coord2 { x, y }, tile.clone());
            }
        }
    }

    /// Prints a grapheme identifier-encoded text using some style options like
    /// ratio to the screen.
    pub fn styled_text(
        &mut self,
        gstring: &GString,
        style: Style,
    ) -> Result<Nat> {
        let mut len = gstring.count_graphemes();
        let mut slice = gstring.index(..);
        let screen_size = self.handle.screen_size();
        let size = style.make_size(screen_size);

        let mut cursor = Coord2 { x: 0, y: style.top_margin };
        let mut is_inside = cursor.y - style.top_margin < size.y;

        while len > 0 && is_inside {
            is_inside = cursor.y - style.top_margin + 1 < size.y;
            let pos = if size.x as usize <= slice.len() {
                let mut pos = slice
                    .index(.. size.x as usize)
                    .iter()
                    .rev()
                    .position(|grapheme| grapheme == Grapheme::space())
                    .map_or(size.x as usize, |rev| len - rev);
                if !is_inside {
                    pos -= 1;
                }
                pos
            } else {
                len
            };
            cursor.x = size.x - pos as Nat;
            cursor.x = cursor.x + style.left_margin - style.right_margin;
            cursor.x = cursor.x / style.den * style.num;
            for grapheme in &slice.index(.. pos) {
                let tile =
                    Tile { grapheme: grapheme.clone(), colors: style.colors };
                self.set(cursor, tile);
                cursor.x += 1;
            }

            if pos != len && !is_inside {
                let tile = Tile {
                    grapheme: Grapheme::new_lossy("…"),
                    colors: style.colors,
                };
                self.set(cursor, tile);
            }

            slice = slice.index(pos ..);
            cursor.y += 1;
            len -= pos;
        }
        Ok(cursor.y)
    }

    async fn resize(&mut self, size: Coord2<Nat>) -> Result<()> {
        let mut stdout = self.handle.shared.stdout.lock().await;
        let buf = format!("{}", terminal::Clear(terminal::ClearType::All));
        write_and_flush(buf.as_bytes(), &mut stdout).await?;
        self.contents.clear(size);

        self.handle
            .shared
            .screen_size
            .store(size.x as u32 | (size.y as u32) << 16, Release);
        Ok(())
    }
}

#[derive(Debug)]
struct Shared {
    cleanedup: AtomicBool,
    min_screen: Coord2<Nat>,
    event_chan: Mutex<watch::Receiver<Event>>,
    stdout: Mutex<io::Stdout>,
    screen_size: AtomicU32,
    screen_contents: Mutex<ScreenContents>,
    frame_rate: Duration,
}

impl Drop for Shared {
    fn drop(&mut self) {
        if !self.cleanedup.load(Relaxed) {
            let _ = terminal::disable_raw_mode();
            let mut buf = String::new();
            let _: Result<()> = write!(buf, "{}", cursor::Show)
                .map_err(Into::into)
                .and_then(|_| restore_screen(&mut buf))
                .map_err(Into::into)
                .map(|_| println!("{}", buf));
        }
    }
}

/// Happens when the event listener fails and disconnects.
#[derive(Debug, Clone, Default)]
pub struct ListenerFailed;

impl ErrorExt for ListenerFailed {}

impl fmt::Display for ListenerFailed {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Event listener failed and disconnected")
    }
}

async fn write_and_flush<'guard>(
    buf: &[u8],
    stdout: &mut MutexGuard<'guard, io::Stdout>,
) -> Result<()> {
    stdout.write_all(buf).await?;
    stdout.flush().await?;
    Ok(())
}

/// Saves the screen previous the application.
#[cfg(windows)]
fn save_screen(buf: &mut String) -> Result<()> {
    if terminal::EnterAlternateScreen.is_ansi_code_supported() {
        write!(buf, "{}", terminal::EnterAlternateScreen.ansi_code())?;
    }
    Ok(())
}

/// Saves the screen previous the application.
#[cfg(unix)]
fn save_screen(buf: &mut String) -> Result<()> {
    write!(buf, "{}", terminal::EnterAlternateScreen.ansi_code())?;
    Ok(())
}

/// Restores the screen previous the application.
#[cfg(windows)]
fn restore_screen(buf: &mut String) -> Result<()> {
    if terminal::LeaveAlternateScreen.is_ansi_code_supported() {
        write!(buf, "{}", terminal::LeaveAlternateScreen.ansi_code())?;
    }
    Ok(())
}

/// Restores the screen previous the application.
#[cfg(unix)]
fn restore_screen(buf: &mut String) -> Result<()> {
    write!(buf, "{}", terminal::LeaveAlternateScreen.ansi_code())?;
    Ok(())
}
