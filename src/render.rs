use crate::orient::{Coord, Coord2D};

/// Minimum size supported for screen.
pub const MIN_SCREEN: Coord2D = Coord2D { x: 80, y: 24 };

/*
/// The context of a draw, including offset, area, screen position, error, etc.
#[derive(Debug)]
pub struct Context<'output, B>
where
    B: Backend,
{
    /// The portion of the object to be drawn.
    pub crop: Rect,
    /// The dimensions of the screen.
    pub screen: Coord2D,
    /// Where it is being drawn.
    pub cursor: Coord2D,
    /// A possible error found when writing.
    pub error: &'output mut GameResult<()>,
    /// The backend to which everything will be written.
    pub term: &'output mut Terminal<B>,
}

impl<'output, B> Context<'output, B>
where
    B: Backend,
{
    /// Creates a new context.
    pub fn new(
        error: &'output mut GameResult<()>,
        term: &'output mut Terminal<B>,
        crop: Rect,
        screen: Coord2D,
    ) -> Self {
        Self { error, term, crop, screen, cursor: Coord2D { x: 0, y: 0 } }
    }

    /// Creates a new context that only draws at a rectangle of this context.
    pub fn sub_context<'sub>(&'sub mut self, crop: Rect) -> Context<'sub, B> {
        Context {
            crop,
            screen: Coord2D::from_map(|axis| {
                crop.start[axis] + self.screen[axis] - self.crop.start[axis]
            }),
            cursor: Coord2D { x: 0, y: 0 },
            error: self.error,
            term: self.term,
        }
    }

    /// Handles the given result and sets internal error output to the found
    /// error, if any.
    pub fn fail(&mut self, result: GameResult<()>) -> fmt::Result {
        result.map_err(|error| {
            if self.error.is_ok() {
                *self.error = Err(error);
            }
            fmt::Error
        })
    }

    fn goto_cursor(&mut self) -> fmt::Result {
        let res = self.term.goto(Coord2D::from_map(|axis| {
            self.cursor[axis] + self.screen[axis] - self.crop.start[axis]
        }));
        self.fail(res)
    }

    fn write_raw(&mut self, grapheme: &str) -> fmt::Result {
        let res = self.term.write_all(grapheme.as_bytes());
        self.fail(res.map_err(Into::into))
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

/// Core implementation of renderization for types that can be rendered on a
/// screen.
pub trait RenderCore {
    /// Renders self on the screen managed by the passed backend.
    fn render_raw<B>(&self, ctx: &mut Context<B>) -> fmt::Result
    where
        B: Backend;

    /// Clears some previous rendering area.
    fn clear_raw<B>(&self, ctx: &mut Context<B>) -> fmt::Result
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

/// Renderization for types that can be rendered on a screen in a given
/// position.
pub trait Render: RenderCore + Positioned {
    /// Renders self on the screen managed by the passed backend.
    fn render<B>(
        &self,
        camera: Camera,
        term: &mut Terminal<B>,
    ) -> GameResult<bool>
    where
        B: Backend,
    {
        /*let node = map.at(self.top_left());
        let mut err = Ok(());
        if let Some(mut ctx) = camera.make_context(node, &mut err, term) {
            let _ = self.render_raw(&mut ctx);
            err?;
            Ok(true)
        } else {
            err?;
            Ok(false)
        }*/

        unimplemented!()
    }

    /// Clears some previous rendering area.
    fn clear<B>(
        &self,
        camera: Camera,
        term: &mut Terminal<B>,
    ) -> GameResult<bool>
    where
        B: Backend,
    {
        /*let node = map.at(self.top_left());
        let mut err = Ok(());
        if let Some(mut ctx) = camera.make_context(node, &mut err, term) {
            let _ = self.clear_raw(&mut ctx);
            err?;
            Ok(true)
        } else {
            err?;
            Ok(false)
        }*/

        unimplemented!()
    }
}

impl<T> Render for T where T: RenderCore + Positioned {}
*/

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
    DarkGrey,
    /// A light gray or intense white color.
    LightGrey,
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

/// Alignment and margin settings for texts.
#[derive(Debug, Clone, Copy)]
pub struct TextSettings {
    /// Left margin.
    pub lmargin: Coord,
    /// Right margin.
    pub rmargin: Coord,
    /// Alignment numerator.
    pub num: Coord,
    /// Alignment denominator.
    pub den: Coord,
}

impl Default for TextSettings {
    fn default() -> Self {
        Self { lmargin: 0, rmargin: 0, num: 0, den: 1 }
    }
}

impl TextSettings {
    /// Default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets left margin.
    pub fn lmargin(self, lmargin: Coord) -> Self {
        Self { lmargin, ..self }
    }

    /// Sets right margin.
    pub fn rmargin(self, rmargin: Coord) -> Self {
        Self { rmargin, ..self }
    }

    /// Sets alignment. Numerator and denominator are used such that
    /// line\[index\] * num / den == screen\[index\]
    pub fn align(self, num: Coord, den: Coord) -> Self {
        Self { num, den, ..self }
    }
}
