mod termion;

pub use self::termion::Termion;
use crate::{
    key::Key,
    orient::{Coord, Coord2D, Direc},
    render::Color,
};
use std::io;

/// An adapter to a terminal backend.
pub trait Backend: Sized + io::Write {
    /// Loads the backend adapter.
    fn load() -> io::Result<Self>;

    /// Awaits for a key to be pressed and returns such key.
    fn wait_key(&mut self) -> io::Result<Key> {
        loop {
            if let Some(key) = self.try_get_key()? {
                break Ok(key);
            }
        }
    }

    /// Checks if there is a pressed key and returns it. If no key has been
    /// pressed, None is returned.
    fn try_get_key(&mut self) -> io::Result<Option<Key>>;

    /// Moves the cursor to the specified 0-based coordinates. An error is
    /// returned if coordinates are outside screen.
    fn goto(&mut self, point: Coord2D) -> io::Result<()>;

    /// Moves the cursor to the specified direction by the given count of steps.
    /// An error is returned if resulting coordinates are outside screen.
    fn move_rel(&mut self, direc: Direc, count: Coord) -> io::Result<()>;

    /// Returns the size of the terminal.
    fn term_size(&mut self) -> io::Result<Coord2D>;

    /// Set the background color to the specified color.
    fn setbg(&mut self, color: Color) -> io::Result<()>;

    /// Set the foreground color to the specified color.
    fn setfg(&mut self, color: Color) -> io::Result<()>;

    /// Clears the whole screen.
    fn clear_screen(&mut self) -> io::Result<()> {
        let size = self.term_size()?;

        for y in 0 .. size.y {
            self.goto(Coord2D { x: 0, y })?;
            for _ in 0 .. size.x {
                write!(self, " ")?
            }
        }

        Ok(())
    }
}
