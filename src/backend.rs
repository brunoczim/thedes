mod termion;

pub use self::termion::Termion;
use crate::{
    error::GameResult,
    key::Key,
    orient::{Coord, Coord2D, Direc},
    render::{Color, TextSettings},
};
use std::io;
use unicode_segmentation::UnicodeSegmentation;

pub type DefaultBackend = Termion;

/// An adapter to a terminal backend.
pub trait Backend: Sized + io::Write {
    /// Loads the backend adapter.
    fn load() -> GameResult<Self>;

    /// Awaits for a key to be pressed and returns such key.
    fn wait_key(&mut self) -> GameResult<Key> {
        loop {
            if let Some(key) = self.try_get_key()? {
                break Ok(key);
            }
        }
    }

    /// Checks if there is a pressed key and returns it. If no key has been
    /// pressed, None is returned.
    fn try_get_key(&mut self) -> GameResult<Option<Key>>;

    /// Moves the cursor to the specified 0-based coordinates. An error is
    /// returned if coordinates are outside screen.
    fn goto(&mut self, point: Coord2D) -> GameResult<()>;

    /// Moves the cursor to the specified direction by the given count of steps.
    /// An error is returned if resulting coordinates are outside screen.
    fn move_rel(&mut self, direc: Direc, count: Coord) -> GameResult<()>;

    /// Returns the size of the terminal.
    fn screen_size(&mut self) -> GameResult<Coord2D>;

    /// Set the background color to the specified color.
    fn setbg(&mut self, color: Color) -> GameResult<()>;

    /// Set the foreground color to the specified color.
    fn setfg(&mut self, color: Color) -> GameResult<()>;

    /// Clears the whole screen.
    fn clear_screen(&mut self) -> GameResult<()> {
        let size = self.screen_size()?;

        for y in 0 .. size.y {
            self.goto(Coord2D { x: 0, y })?;
            for _ in 0 .. size.x {
                write!(self, " ")?
            }
        }

        Ok(())
    }

    /// Writes a multiline text with settings (including alignment from a given
    /// ratio).  The ratio is used such that `line[len * ratio] == screen[len *
    /// ratio]`.  Returns the index of the line after the text.
    fn text(
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
        let screen = self.screen_size()?;
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
}
