use std::{
    collections::BTreeSet,
    fmt::{self, Write as _},
    io::{self, Write as _},
    mem,
};

use crossterm::{
    cursor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    Command,
};
use thedes_geometry::rect;
use thiserror::Error;

use crate::{
    color::{self, Color, ColorPair, Mutation as _},
    geometry::{Coord, CoordPair},
    grapheme::{self, NotGrapheme},
    tile::{self, Tile},
    Config,
};

pub use style::TextStyle;

mod style;

#[derive(Debug, Error)]
#[error("Point is outside of screen canvas rectangle")]
pub struct InvalidCanvasPoint {
    #[from]
    source: rect::HorzAreaError<usize>,
}

#[derive(Debug, Error)]
#[error("Index is outside of screen canvas buffer bounds")]
pub struct InvalidCanvasIndex {
    #[from]
    source: rect::InvalidArea<usize>,
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to render commands")]
    Fmt(
        #[from]
        #[source]
        fmt::Error,
    ),
    #[error("Inconsistent canvas rectangle point manipulation")]
    InvalidCanvasPoint(
        #[from]
        #[source]
        InvalidCanvasPoint,
    ),
    #[error("Inconsistent canvas index manipulation")]
    InvalidCanvasIndex(
        #[from]
        #[source]
        InvalidCanvasIndex,
    ),
    #[error("Inconsistent grapheme ID")]
    UnknwonGraphemeId(
        #[from]
        #[source]
        grapheme::UnknownId,
    ),
    #[error("Inconsistent grapheme generation")]
    NotGrapheme(
        #[from]
        #[source]
        NotGrapheme,
    ),
    #[error("Could not flush stdout")]
    Flush(#[source] io::Error),
}

#[derive(Debug)]
pub struct Screen {
    term_size: CoordPair,
    canvas_size: CoordPair,
    default_colors: ColorPair,
    render_buf: String,
    current_colors: ColorPair,
    current_position: CoordPair,
    working_buf: Box<[Tile]>,
    displayed_buf: Box<[Tile]>,
    dirty: BTreeSet<CoordPair>,
    grapheme_registry: grapheme::Registry,
}

impl Screen {
    pub(crate) fn new(
        config: &Config,
        term_size: CoordPair,
    ) -> Result<Self, RenderError> {
        let canvas_size = config.canvas_size();
        let default_colors = config.default_colors();
        let mut grapheme_registry = grapheme::Registry::new();
        let space = grapheme_registry.get_or_register(" ")?;
        let tile_buf_size =
            usize::from(canvas_size.x) * usize::from(canvas_size.y);
        let tile_buf = Box::<[Tile]>::from(vec![
            Tile {
                grapheme: space,
                colors: default_colors,
            };
            tile_buf_size
        ]);

        let mut this = Self {
            term_size,
            canvas_size,
            default_colors,
            render_buf: String::new(),
            current_colors: default_colors,
            current_position: CoordPair { x: 0, y: 0 },
            working_buf: tile_buf.clone(),
            displayed_buf: tile_buf,
            dirty: BTreeSet::new(),
            grapheme_registry,
        };

        this.enter()?;
        this.term_size_changed(term_size)?;

        Ok(this)
    }

    pub fn needs_resize(&self) -> bool {
        self.canvas_size
            .zip2(self.term_size)
            .any(|(canvas, term)| canvas >= term)
    }

    pub fn canvas_size(&self) -> CoordPair {
        self.canvas_size
    }

    pub fn get(
        &self,
        canvas_point: CoordPair,
    ) -> Result<Tile, InvalidCanvasPoint> {
        let index = self.point_to_index(canvas_point)?;
        Ok(self.working_buf[index])
    }

    pub fn set(
        &mut self,
        canvas_point: CoordPair,
        tile: Tile,
    ) -> Result<Tile, InvalidCanvasPoint> {
        let index = self.point_to_index(canvas_point)?;
        if tile == self.displayed_buf[index] {
            self.dirty.remove(&canvas_point);
        } else {
            self.dirty.insert(canvas_point);
        }
        let old = mem::replace(&mut self.working_buf[index], tile);
        Ok(old)
    }

    pub fn mutate<M>(
        &mut self,
        canvas_point: CoordPair,
        mutation: M,
    ) -> Result<Tile, InvalidCanvasPoint>
    where
        M: tile::Mutation,
    {
        let tile = self.get(canvas_point)?;
        let old = self.set(canvas_point, mutation.mutate_tile(tile))?;
        Ok(old)
    }

    pub fn clear_canvas(
        &mut self,
        background: Color,
    ) -> Result<(), RenderError> {
        let space = self.grapheme_registry.get_or_register(" ")?;
        for y in 0 .. self.canvas_size().y {
            for x in 0 .. self.canvas_size().x {
                self.mutate(CoordPair { x, y }, |tile: Tile| Tile {
                    colors: ColorPair { background, ..tile.colors },
                    grapheme: space,
                })?;
            }
        }
        Ok(())
    }

    pub fn styled_text(
        &mut self,
        input: &str,
        style: &TextStyle,
    ) -> Result<Coord, RenderError> {
        let graphemes: Vec<_> =
            self.grapheme_registry.get_or_register_many(input).collect();
        let mut slice = &graphemes[..];
        let canvas_size = self.canvas_size;
        let size = style.make_size(canvas_size);

        let mut cursor = CoordPair { x: 0, y: style.top_margin() };
        let mut is_inside = cursor.y - style.top_margin() < size.y;

        while !slice.is_empty() && is_inside {
            is_inside = cursor.y - style.top_margin() + 1 < size.y;
            let width = usize::from(size.x);
            let pos = self.find_break_pos(width, size, slice, is_inside)?;

            cursor.x = size.x - pos as Coord;
            cursor.x = cursor.x + style.left_margin() - style.right_margin();
            cursor.x = cursor.x * style.align_numer() / style.align_denom();

            let (low, high) = slice.split_at(pos);
            slice = high;

            self.print_slice(low, &style, &mut cursor)?;

            if pos != slice.len() && !is_inside {
                let elipsis = self.grapheme_registry.get_or_register("…")?;
                self.mutate(cursor, |tile: Tile| {
                    let colors = style.colors().mutate_colors(tile.colors);
                    Tile { grapheme: elipsis, colors }
                })?;
            }

            cursor.y += 1;
        }

        Ok(cursor.y)
    }

    pub fn grapheme_registry(&self) -> &grapheme::Registry {
        &self.grapheme_registry
    }

    pub fn grapheme_registry_mut(&mut self) -> &mut grapheme::Registry {
        &mut self.grapheme_registry
    }

    pub(crate) fn render(&mut self) -> Result<(), RenderError> {
        if !self.needs_resize() {
            self.draw_working_canvas()?;
            self.flush()?;
        }
        Ok(())
    }

    pub(crate) fn term_size_changed(
        &mut self,
        new_term_size: CoordPair,
    ) -> Result<(), RenderError> {
        self.move_to_origin()?;
        self.change_colors(self.default_colors)?;
        self.dirty.clear();
        let space = self.grapheme_registry.get_or_register(" ")?;
        for (i, (working, displayed)) in self
            .working_buf
            .iter_mut()
            .zip(&mut self.displayed_buf[..])
            .enumerate()
        {
            *displayed = Tile { grapheme: space, colors: self.default_colors };
            if *displayed != *working {
                let point = self
                    .canvas_size
                    .map(usize::from)
                    .as_rect_size(thedes_geometry::CoordPair::from_axes(|_| 0))
                    .checked_bot_right_of_horz_area(&i)
                    .map_err(InvalidCanvasIndex::from)?
                    .map(|a| a as Coord);
                self.dirty.insert(point);
            }
        }

        self.term_size = new_term_size;
        if self.needs_resize() {
            self.draw_resize_msg()?;
        } else {
            self.draw_reset()?;
        }
        self.flush()?;
        Ok(())
    }

    /// Finds the position where a line should break in a styled text.
    fn find_break_pos(
        &mut self,
        width: usize,
        box_size: CoordPair,
        graphemes: &[grapheme::Id],
        is_inside: bool,
    ) -> Result<usize, RenderError> {
        let space = self.grapheme_registry.get_or_register(" ")?;
        if width <= graphemes.len() {
            let mut pos = graphemes[.. usize::from(box_size.x)]
                .iter()
                .rposition(|grapheme| *grapheme == space)
                .unwrap_or(width);
            if !is_inside {
                pos -= 1;
            }
            Ok(pos)
        } else {
            Ok(graphemes.len())
        }
    }

    /// Writes a slice using the given style. It should fit in one line.
    fn print_slice<C>(
        &mut self,
        slice: &[grapheme::Id],
        style: &TextStyle<C>,
        cursor: &mut CoordPair,
    ) -> Result<(), RenderError>
    where
        C: color::Mutation,
    {
        for grapheme in slice {
            self.mutate(*cursor, |tile: Tile| Tile {
                grapheme: *grapheme,
                colors: style.colors().mutate_colors(tile.colors),
            })?;
            cursor.x += 1;
        }

        Ok(())
    }

    fn enter(&mut self) -> Result<(), RenderError> {
        EnterAlternateScreen.write_ansi(&mut self.render_buf)?;
        write!(self.render_buf, "{}", cursor::Hide)?;
        self.flush()?;
        Ok(())
    }

    fn leave(&mut self) -> Result<(), RenderError> {
        write!(self.render_buf, "{}", cursor::Show)?;
        write!(
            self.render_buf,
            "{}",
            crossterm::style::SetBackgroundColor(
                crossterm::style::Color::Reset
            )
        )?;
        write!(
            self.render_buf,
            "{}",
            crossterm::style::SetForegroundColor(
                crossterm::style::Color::Reset
            )
        )?;
        LeaveAlternateScreen.write_ansi(&mut self.render_buf)?;
        self.flush()?;
        Ok(())
    }

    fn clear_term(&mut self, background: Color) -> Result<(), RenderError> {
        if background != self.current_colors.background {
            self.change_background(background)?;
        }
        write!(
            self.render_buf,
            "{}",
            terminal::Clear(terminal::ClearType::All)
        )?;
        Ok(())
    }

    fn move_to(&mut self, term_point: CoordPair) -> Result<(), RenderError> {
        write!(
            self.render_buf,
            "{}",
            cursor::MoveTo(term_point.x, term_point.y)
        )?;
        self.current_position = term_point;
        Ok(())
    }

    fn move_to_origin(&mut self) -> Result<(), RenderError> {
        self.move_to(CoordPair { x: 0, y: 0 })
    }

    fn change_foreground(&mut self, color: Color) -> Result<(), RenderError> {
        write!(
            self.render_buf,
            "{}",
            crossterm::style::SetForegroundColor(color.to_crossterm()),
        )?;
        self.current_colors.foreground = color;
        Ok(())
    }

    fn change_background(&mut self, color: Color) -> Result<(), RenderError> {
        write!(
            self.render_buf,
            "{}",
            crossterm::style::SetBackgroundColor(color.to_crossterm()),
        )?;
        self.current_colors.background = color;
        Ok(())
    }

    fn change_colors(&mut self, colors: ColorPair) -> Result<(), RenderError> {
        self.change_foreground(colors.foreground)?;
        self.change_background(colors.background)?;
        Ok(())
    }

    fn draw_grapheme(&mut self, id: grapheme::Id) -> Result<(), RenderError> {
        let grapheme = self.grapheme_registry.lookup(id)?;
        write!(self.render_buf, "{}", grapheme)?;
        self.current_position.x += 1;
        if self.current_position.x == self.canvas_size.x {
            self.current_position.x = 0;
            self.current_position.y += 1;
            if self.current_position.y == self.canvas_size.y {
                self.move_to_origin()?;
            }
        }
        Ok(())
    }

    fn draw_tile(
        &mut self,
        term_point: CoordPair,
        tile: Tile,
    ) -> Result<(), RenderError> {
        if self.current_position != term_point {
            self.move_to(term_point)?;
        }
        if self.current_colors.foreground != tile.colors.foreground {
            self.change_foreground(tile.colors.foreground)?;
        }
        if self.current_colors.background != tile.colors.background {
            self.change_background(tile.colors.background)?;
        }
        self.draw_grapheme(tile.grapheme)?;
        Ok(())
    }

    fn draw_reset_hor_line(
        &mut self,
        y: Coord,
        x_start: Coord,
        x_end: Coord,
    ) -> Result<(), RenderError> {
        let tile = Tile {
            colors: self.default_colors,
            grapheme: self.grapheme_registry.get_or_register("━")?,
        };
        for x in x_start .. x_end {
            self.draw_tile(CoordPair { x, y }, tile)?;
        }
        Ok(())
    }

    fn draw_reset(&mut self) -> Result<(), RenderError> {
        self.move_to_origin()?;
        self.clear_term(self.default_colors.background)?;

        let margin_top_left = self.top_left_margin();
        let margin_bottom_right = self.bottom_right_margin();

        let tile = Tile {
            grapheme: self.grapheme_registry.get_or_register("┏")?,
            colors: self.default_colors,
        };
        self.draw_tile(margin_top_left - 1, tile)?;
        self.draw_reset_hor_line(
            margin_top_left.y - 1,
            margin_top_left.x,
            margin_bottom_right.x,
        )?;
        let tile = Tile {
            grapheme: self.grapheme_registry.get_or_register("┓")?,
            colors: self.default_colors,
        };
        self.draw_tile(
            CoordPair { x: margin_bottom_right.x, y: margin_top_left.y - 1 },
            tile,
        )?;

        let tile = Tile {
            grapheme: self.grapheme_registry.get_or_register("┃")?,
            colors: self.default_colors,
        };
        for y in margin_top_left.y .. margin_bottom_right.y {
            self.draw_tile(CoordPair { x: margin_top_left.x - 1, y }, tile)?;
            self.draw_tile(CoordPair { x: margin_bottom_right.x, y }, tile)?;
        }

        let tile = Tile {
            grapheme: self.grapheme_registry.get_or_register("┗")?,
            colors: self.default_colors,
        };
        self.draw_tile(
            CoordPair { x: margin_top_left.x - 1, y: margin_bottom_right.y },
            tile,
        )?;
        self.draw_reset_hor_line(
            margin_bottom_right.y,
            margin_top_left.x,
            margin_bottom_right.x,
        )?;
        let tile = Tile {
            grapheme: self.grapheme_registry.get_or_register("┛")?,
            colors: self.default_colors,
        };
        self.draw_tile(margin_bottom_right, tile)?;

        Ok(())
    }

    fn draw_resize_msg(&mut self) -> Result<(), RenderError> {
        let graphemes: Vec<_> = self
            .grapheme_registry
            .get_or_register_many(&format!(
                "RESIZE {}x{}",
                self.canvas_size.x + 2,
                self.canvas_size.y + 2
            ))
            .collect();
        self.move_to_origin()?;
        self.clear_term(self.default_colors.background)?;
        for (i, grapheme) in graphemes.into_iter().enumerate() {
            self.draw_tile(
                CoordPair { x: i as Coord, y: 0 },
                Tile { colors: self.default_colors, grapheme },
            )?;
        }
        Ok(())
    }

    fn draw_working_canvas(&mut self) -> Result<(), RenderError> {
        for canvas_point in mem::take(&mut self.dirty) {
            let tile = self.get(canvas_point)?;
            let term_point = self.canvas_to_term(canvas_point);
            self.draw_tile(term_point, tile)?;
        }
        self.displayed_buf.clone_from(&self.working_buf);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), RenderError> {
        print!("{}", self.render_buf);
        self.render_buf.clear();
        io::stdout().flush().map_err(RenderError::Flush)?;
        Ok(())
    }

    fn top_left_margin(&self) -> CoordPair {
        (self.term_size - self.canvas_size) / 2 + 1
    }

    fn bottom_right_margin(&self) -> CoordPair {
        self.top_left_margin() + self.canvas_size
    }

    fn point_to_index(
        &self,
        canvas_point: CoordPair,
    ) -> Result<usize, InvalidCanvasPoint> {
        let index = self
            .canvas_size
            .map(usize::from)
            .as_rect_size(thedes_geometry::CoordPair::from_axes(|_| 0))
            .checked_horz_area_down_to(canvas_point.map(usize::from))?;
        Ok(index)
    }

    fn canvas_to_term(&self, canvas_point: CoordPair) -> CoordPair {
        canvas_point + self.top_left_margin()
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.leave().expect("should leave alternate screen")
    }
}
