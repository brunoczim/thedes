mod termion;

pub use self::termion::Termion;
use crate::{key::Key, tile::Color};
use std::io;

/// Types that can be rendered on a screen.
pub trait Render {
    /// Renders self on the screen managed by the passed backend.
    fn render<B>(&self, backend: &mut B) -> io::Result<()>
    where
        B: Backend;
}

/// An adapter to a terminal backend.
pub trait Backend: Sized + io::Write {
    /// Loads the backend adapter.
    fn load() -> io::Result<Self>;

    /// Awaits for a key to be pressed and returns such key.
    fn wait_key(&mut self) -> io::Result<Key>;

    /// Checks if there is a pressed key and returns it. If no key has been
    /// pressed, None is returned.
    fn try_get_key(&mut self) -> io::Result<Option<Key>>;

    /// Moves the cursor to the specified coordinates. An error is returned
    /// if coordinates are outside screen.
    fn goto(&mut self, x: usize, y: usize) -> io::Result<()>;

    /// Set the background color to the specified color.
    fn setbg(&mut self, color: Color) -> io::Result<()>;

    /// Set the foreground color to the specified color.
    fn setfg(&mut self, color: Color) -> io::Result<()>;
}
