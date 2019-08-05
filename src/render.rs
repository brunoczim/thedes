use crate::{backend::Backend, orient::{Coord, Direc}};
use std::io;

/// Types that can be rendered on a screen.
pub trait Render {
    /// Renders self on the screen managed by the passed backend.
    fn render<B>(&self, backend: &mut B) -> io::Result<()>
    where
        B: Backend;
}

/// A supported color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// The black color.
    Black,
    /// The white color.
    White,
    /// The red color.
    Red,
    /// The green color.
    Green,
    /// The blue color.
    Blue,
    /// The magenta color.
    Magenta,
    /// The yellow color.
    Yellow,
    /// The cyan color.
    Cyan,
    /// A dark gray or intense black color.
    LightBlack,
    /// A light gray or intense white color.
    LightWhite,
    /// A light intense red color.
    LightRed,
    /// A light intense green color.
    LightGreen,
    /// A light intense blue color.
    LightBlue,
    /// A light intense magenta color.
    LightMagenta,
    /// A light intense magenta color.
    LightYellow,
    /// A light intense cyan color.
    LightCyan,
}

/// A segment of a graphical component.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Segment {
    /// Unicode chars used as graphics
    Unicode {
        /// Chars of the graphics
        chars: Box<str>,
        /// Foreground color of the graphics
        fg: Color,
        /// Background color of the graphics
        bg: Color,
    },
    /// Like a newline, but moves the cursor to the lefmost position of
    /// the graphics.
    Endline,
    /// Skips n cells when writing.
    Skip(Coord),
}

/// A graphics sprite
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sprite {
    /// The segments composing this sprite
    pub segments: Box<[Segment]>,
}

impl Render for Sprite {
    fn render<B>(&self, backend: &mut B) -> io::Result<()>
    where
        B: Backend,
    {
        let (x, mut y) = backend.pos()?;

        for segment in &*self.segments {
            match segment {
                Segment::Unicode { chars, fg, bg } => {
                    backend.setfg(*fg)?;
                    backend.setbg(*bg)?;
                    write!(backend, "{}", chars)?;
                },

                Segment::Endline => {
                    y += 1;
                    backend.goto(x, y)?;
                },

                Segment::Skip(count) => {
                    backend.move_rel(Direc::Right, *count)?;
                }
            }
        }

        Ok(())
    }
}
