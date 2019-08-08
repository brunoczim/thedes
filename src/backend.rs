mod termion;

pub use self::termion::Termion;
use crate::{
    key::Key,
    orient::{Coord, Direc},
    render::Color,
};
use std::io;

/// An adapter to a terminal backend.
pub trait Backend: Sized + io::Write {
    /// Loads the backend adapter.
    fn load() -> io::Result<Self>;

    /// Awaits for a key to be pressed and returns such key.
    fn wait_key(&mut self) -> io::Result<Key>;

    /// Checks if there is a pressed key and returns it. If no key has been
    /// pressed, None is returned.
    fn try_get_key(&mut self) -> io::Result<Option<Key>>;

    /// Moves the cursor to the specified 0-based coordinates. An error is returned
    /// if coordinates are outside screen.
    fn goto(&mut self, x: Coord, y: Coord) -> io::Result<()>;

    /// Moves the cursor to the specified direction by the given count of steps. An error is returned
    /// if resulting coordinates are outside screen.
    fn move_rel(&mut self, direc: Direc, count: Coord) -> io::Result<()>;

    /// Returns the 0-based position (x,y) of a cursor on the screen.
    fn pos(&mut self) -> io::Result<(Coord, Coord)>;

    /// Set the background color to the specified color.
    fn setbg(&mut self, color: Color) -> io::Result<()>;

    /// Set the foreground color to the specified color.
    fn setfg(&mut self, color: Color) -> io::Result<()>;
}
