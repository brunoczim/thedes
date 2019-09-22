use crate::{
    backend::Backend,
    orient::{Coord2D, Rect},
};
use std::{
    fmt::{self, Write},
    io,
};
use unicode_segmentation::UnicodeSegmentation;

/// The context of a draw, including offset, area, screen position, error, etc.
#[derive(Debug)]
pub struct Context<'output, B> {
    crop: Rect,
    screen: Coord2D,
    cursor: Coord2D,
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
        error: &'output mut io::Result<()>,
        backend: &'output mut B,
        crop: Rect,
        screen: Coord2D,
    ) -> Self {
        Self { error, backend, crop, screen, cursor: Coord2D { x: 0, y: 0 } }
    }

    /// Creates a new context that only draws at a rectangle of this context.
    pub fn sub_context<'sub>(&'sub mut self, crop: Rect) -> Context<'sub, B> {
        Context {
            crop,
            screen: Coord2D::from_map(|axis| {
                crop.start[axis] - self.crop.start[axis] + self.screen[axis]
            }),
            cursor: Coord2D { x: 0, y: 0 },
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
        let res = self.backend.goto(Coord2D::from_map(|axis| {
            self.cursor[axis] - self.crop.start[axis] + self.screen[axis]
        }));
        self.fail(res)
    }

    fn write_raw(&mut self, grapheme: &str) -> fmt::Result {
        let res = self.backend.write_all(grapheme.as_bytes());
        self.fail(res)
    }

    fn jump_line(&mut self) {
        self.cursor.y += 1;
        self.cursor.x = 0;
    }

    fn write_grapheme(&mut self, grapheme: &str) -> fmt::Result {
        if self.crop.has_point(self.cursor) {
            self.goto_cursor()?;
            self.write_raw(grapheme)?;
        }

        self.cursor.x += 1;

        Ok(())
    }
}

impl<'sub, B> Write for Context<'sub, B>
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
    fn render<B>(&self, ctx: &mut Context<B>) -> fmt::Result
    where
        B: Backend;

    fn clear<B>(&self, ctx: &mut Context<B>) -> fmt::Result
    where
        B: Backend,
    {
        for _ in 0 .. ctx.crop.end().y {
            for _ in 0 .. ctx.crop.end().x {
                ctx.write_str(" ")?;
            }
            ctx.write_str("\n")?;
        }

        Ok(())
    }
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
