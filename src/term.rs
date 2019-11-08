use crate::{
    backend::{Backend, DefaultBackend},
    error::GameResult,
    key::Key,
    orient::{Coord, Coord2D, Direc},
    render::{Color, TextSettings, MIN_SCREEN},
};
use std::{
    io::{self, Write},
    thread,
    time::{Duration, Instant},
};
use unicode_segmentation::UnicodeSegmentation;

pub use self::Tick::*;

/// A specification on what to do next execution.
pub enum Tick<T> {
    /// Stop and return this value.
    Stop(T),
    /// Continue executing.
    Continue,
}

#[derive(Debug)]
pub struct Terminal<B = DefaultBackend>
where
    B: Backend,
{
    screen_size: Coord2D,
    interval: Duration,
    then: Instant,
    correction: Duration,
    resized: bool,
    backend: B,
}

impl<B> Terminal<B>
where
    B: Backend,
{
    /// Starts an event loop with access to the terminal.
    pub fn start(mut backend: B) -> GameResult<Self> {
        Ok(Self {
            screen_size: backend.screen_size()?,
            interval: Duration::from_millis(50),
            then: Instant::now(),
            correction: Duration::new(0, 0),
            resized: false,
            backend,
        })
    }

    /// Returns the size of the terminal.
    pub fn screen_size(&mut self) -> Coord2D {
        self.screen_size
    }

    /// Returns whether the screen size has been resized.
    pub fn has_resized(&mut self) -> bool {
        let ret = self.resized;
        self.resized = false;
        ret
    }

    /// Checks if there is a pressed key and returns it. If no key has been
    /// pressed, None is returned.
    pub fn key(&mut self) -> GameResult<Option<Key>> {
        self.backend.try_get_key()
    }

    /// Set the background color to the specified color.
    pub fn setbg(&mut self, color: Color) -> GameResult<()> {
        self.backend.setbg(color)
    }

    /// Set the foreground color to the specified color.
    pub fn setfg(&mut self, color: Color) -> GameResult<()> {
        self.backend.setfg(color)
    }

    /// Clears the whole screen.
    pub fn clear_screen(&mut self) -> GameResult<()> {
        self.backend.clear_screen()
    }

    /// Moves the cursor to the specified 0-based coordinates. An error is
    /// returned if coordinates are outside screen.
    pub fn goto(&mut self, point: Coord2D) -> GameResult<()> {
        self.backend.goto(point)
    }

    /// Moves the cursor to the specified direction by the given count of steps.
    /// An error is returned if resulting coordinates are outside screen.
    pub fn move_rel(&mut self, direc: Direc, count: Coord) -> GameResult<()> {
        self.backend.move_rel(direc, count)
    }

    /// Writes a multiline text with settings (including alignment from a given
    /// ratio).  The ratio is used such that `line[len * ratio] == screen[len *
    /// ratio]`.  Returns the index of the line after the text.
    pub fn text(
        &mut self,
        string: &str,
        y: Coord,
        settings: TextSettings,
    ) -> GameResult<Coord> {
        let mut indices =
            string.grapheme_indices(true).map(|(i, _)| i).collect::<Vec<_>>();
        indices.push(string.len());
        let mut line = 0;
        let mut slice = &*indices;
        let screen = self.screen_size();
        let width = (screen.x - settings.lmargin - settings.rmargin) as usize;

        while slice.len() > 1 {
            let pos = if width >= slice.len() - 1 {
                slice.len() - 1
            } else {
                slice[.. width]
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| i < slice.len() - 1)
                    .rfind(|&(i, &idx)| &string[idx .. slice[i + 1]] == " ")
                    .map_or(width, |(i, _)| i + 1)
            };

            let x = (width - pos) as Coord / settings.den * settings.num;
            self.goto(Coord2D { x, y: y + line })?;
            write!(self, "{}", &string[slice[0] .. slice[pos]])?;
            slice = &slice[pos ..];
            line += 1;
        }
        Ok(y + line)
    }

    /// Calls a function to run on the event loop.
    pub fn call<F, T>(&mut self, mut fun: F) -> GameResult<T>
    where
        F: FnMut(&mut Self) -> GameResult<Tick<T>>,
    {
        loop {
            if let Stop(ret) = self.tick(&mut fun)? {
                break Ok(ret);
            }

            let diff = self.then.elapsed() - self.correction;
            if let Some(time) = self.interval.checked_sub(diff) {
                thread::sleep(time);
                self.correction += time;
            }
        }
    }

    fn tick<F, T>(&mut self, mut fun: F) -> GameResult<Tick<T>>
    where
        F: FnMut(&mut Self) -> GameResult<Tick<T>>,
    {
        self.check_screen_size()?;
        fun(self)
    }

    fn check_screen_size(&mut self) -> GameResult<()> {
        let mut new_screen = self.backend.screen_size()?;

        if new_screen.x < MIN_SCREEN.x || new_screen.y < MIN_SCREEN.y {
            self.clear_screen()?;
            self.goto(Coord2D { x: 0, y: 0 })?;
            write!(self, "RESIZE {:?},{:?}", MIN_SCREEN.x, MIN_SCREEN.y)?;

            while new_screen.x < MIN_SCREEN.x || new_screen.y < MIN_SCREEN.y {
                new_screen = self.backend.screen_size()?;
            }

            self.resized = true;
        }

        if new_screen != self.screen_size {
            self.screen_size = new_screen;
            self.resized = true;
        }
        Ok(())
    }
}

impl<B> Write for Terminal<B>
where
    B: Backend,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.backend.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.backend.flush()
    }
}
