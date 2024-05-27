use std::{
    collections::BTreeSet,
    fmt::{self, Write as _},
    io::{self, Write as _},
    mem,
};

use crossterm::{
    cursor,
    style,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    Command,
};
use thiserror::Error;

use crate::{
    color::{Color, ColorPair},
    geometry::{Coord, CoordPair, InvalidRectPoint},
    grapheme::{self, NotGrapheme},
    tile::{Mutation, Tile},
    Config,
};

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to render commands")]
    Fmt(
        #[from]
        #[source]
        fmt::Error,
    ),
    #[error("Inconsistent point manipulation")]
    InvalidPoint(
        #[from]
        #[source]
        InvalidRectPoint,
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

    pub fn get(
        &self,
        canvas_point: CoordPair,
    ) -> Result<Tile, InvalidRectPoint> {
        let index = self.make_index(canvas_point)?;
        Ok(self.working_buf[index])
    }

    pub fn set(
        &mut self,
        canvas_point: CoordPair,
        tile: Tile,
    ) -> Result<Tile, InvalidRectPoint> {
        let index = self.make_index(canvas_point)?;
        let old = mem::replace(&mut self.working_buf[index], tile);
        Ok(old)
    }

    pub fn mutate<M>(
        &mut self,
        canvas_point: CoordPair,
        mutation: M,
    ) -> Result<Tile, InvalidRectPoint>
    where
        M: Mutation,
    {
        let tile = self.get(canvas_point)?;
        let old = self.set(canvas_point, mutation.mutate_tile(tile))?;
        Ok(old)
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
        self.dirty.clear();
        let space = self.grapheme_registry.get_or_register(" ")?;
        for tile in
            self.working_buf.iter_mut().chain(&mut self.displayed_buf[..])
        {
            *tile = Tile { grapheme: space, colors: self.default_colors };
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
            style::SetBackgroundColor(crossterm::style::Color::Reset)
        )?;
        write!(
            self.render_buf,
            "{}",
            style::SetForegroundColor(crossterm::style::Color::Reset)
        )?;
        LeaveAlternateScreen.write_ansi(&mut self.render_buf)?;
        self.flush()?;
        Ok(())
    }

    fn clear(&mut self, background: Color) -> Result<(), RenderError> {
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
            style::SetForegroundColor(color.to_crossterm()),
        )?;
        Ok(())
    }

    fn change_background(&mut self, color: Color) -> Result<(), RenderError> {
        write!(
            self.render_buf,
            "{}",
            style::SetBackgroundColor(color.to_crossterm()),
        )?;
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
        self.clear(self.default_colors.background)?;

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
                self.canvas_size.x + 1,
                self.canvas_size.y + 1
            ))
            .collect();
        self.move_to_origin()?;
        self.clear(self.default_colors.background)?;
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
        (self.term_size - self.canvas_size) / 2
    }

    fn bottom_right_margin(&self) -> CoordPair {
        self.top_left_margin() + self.canvas_size
    }

    fn make_index(
        &self,
        canvas_point: CoordPair,
    ) -> Result<usize, InvalidRectPoint> {
        self.canvas_size
            .map(usize::from)
            .as_rect_to_line(canvas_point.map(usize::from))
            .map_err(|error| InvalidRectPoint {
                rect_size: error.rect_size.map(|a| a as Coord),
                point: error.point.map(|a| a as Coord),
            })
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
