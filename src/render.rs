use crate::{backend::Backend, orient::Coord};
use std::{fmt, io};
use unicode_segmentation::UnicodeSegmentation;

/// The context of a draw, including offset, area, screen position, error, etc.
#[derive(Debug)]
pub struct Context<'output, B> {
    x: Coord,
    y: Coord,
    width: Coord,
    height: Coord,
    screen_x: Coord,
    screen_y: Coord,
    cursor_x: Coord,
    cursor_y: Coord,
    /// A possible error found when writing.
    pub error: &'output mut io::Result<()>,
    /// The backend to which everything will be written.
    pub backend: &'output mut B,
}

impl<'output, B> Context<'output, B>
where
    B: Backend,
{
    /// Creates a new context.
    pub fn new(
        backend: &'output mut B,
        error: &'output mut io::Result<()>,
        x: Coord,
        y: Coord,
        width: Coord,
        height: Coord,
        screen_x: Coord,
        screen_y: Coord,
    ) -> Self {
        Self {
            error,
            backend,
            x,
            y,
            width,
            height,
            screen_x,
            screen_y,
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    /// Creates a new context that only draws at a rectangle of this context.
    pub fn sub_context<'sub>(
        &'sub mut self,
        x: Coord,
        y: Coord,
        width: Coord,
        height: Coord,
    ) -> Context<'sub, B> {
        Context {
            x,
            y,
            width,
            height,
            screen_x: x - self.x + self.screen_x,
            screen_y: y - self.y + self.screen_y,
            cursor_x: 0,
            cursor_y: 0,
            error: self.error,
            backend: self.backend,
        }
    }

    /// Handles the given result and sets internal error output to the found
    /// error, if any.
    pub fn fail(&mut self, result: io::Result<()>) -> fmt::Result {
        result.map_err(|error| {
            if self.error.is_ok() {
                *self.error = Err(error);
            }
            fmt::Error
        })
    }

    fn goto_cursor(&mut self) -> fmt::Result {
        let res = self.backend.goto(
            self.cursor_x - self.x + self.screen_x,
            self.cursor_y - self.y + self.screen_y,
        );
        self.fail(res)
    }

    fn write_raw(&mut self, grapheme: &str) -> fmt::Result {
        let res = self.backend.write_all(grapheme.as_bytes());
        self.fail(res)
    }

    fn jump_line(&mut self) {
        self.cursor_y += 1;
        self.cursor_x = 0;
    }

    fn write_grapheme(&mut self, grapheme: &str) -> fmt::Result {
        if self.cursor_x >= self.x
            && self.cursor_x < self.x + self.width
            && self.cursor_y >= self.y
            && self.cursor_y < self.y + self.height
        {
            self.goto_cursor()?;
            self.write_raw(grapheme)?;
        }

        self.cursor_x += 1;

        Ok(())
    }
}

impl<'sub, B> fmt::Write for Context<'sub, B>
where
    B: Backend,
{
    fn write_str(&mut self, string: &str) -> fmt::Result {
        if self.error.is_err() {
            return Err(fmt::Error);
        }

        for grapheme in string.graphemes(true) {
            if grapheme == "\n" {
                self.jump_line();
            } else {
                self.write_grapheme(grapheme)?;
            }
        }

        Ok(())
    }
}

/// Types that can be rendered on a screen.
pub trait Render {
    /// Renders self on the screen managed by the passed backend.
    fn render<B>(
        &self,
        x: Coord,
        y: Coord,
        ctx: &mut Context<B>,
    ) -> fmt::Result
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
