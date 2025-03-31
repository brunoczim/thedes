use thedes_tui_core::{
    color::ColorPair,
    geometry::{Coord, CoordPair},
    mutation::{Mutation, Set},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Style<C = Set<ColorPair>>
where
    C: Mutation<ColorPair>,
{
    left_margin: Coord,
    right_margin: Coord,
    top_margin: Coord,
    bottom_margin: Coord,
    min_width: Coord,
    max_width: Coord,
    min_height: Coord,
    max_height: Coord,
    align_numer: Coord,
    align_denom: Coord,
    colors: C,
}

impl Default for Style {
    fn default() -> Self {
        Self::new_with_colors(Set::default())
    }
}

impl<C> Style<C>
where
    C: Mutation<ColorPair>,
{
    pub fn new_with_colors(colors: C) -> Self {
        Self {
            left_margin: 0,
            right_margin: 0,
            top_margin: 0,
            bottom_margin: 0,
            min_width: 0,
            max_width: Coord::max_value(),
            min_height: 0,
            max_height: Coord::max_value(),
            align_numer: 0,
            align_denom: 1,
            colors,
        }
    }

    pub fn map_colors<F, D>(self, mapper: F) -> Style<D>
    where
        F: FnOnce(C) -> D,
        D: Mutation<ColorPair>,
    {
        Style {
            left_margin: self.left_margin,
            right_margin: self.right_margin,
            top_margin: self.top_margin,
            bottom_margin: self.bottom_margin,
            min_width: self.min_width,
            max_width: self.max_width,
            min_height: self.min_height,
            max_height: self.max_height,
            align_numer: self.align_numer,
            align_denom: self.align_denom,
            colors: mapper(self.colors),
        }
    }

    pub fn with_colors<D>(self, colors: D) -> Style<D>
    where
        D: Mutation<ColorPair>,
    {
        self.map_colors(|_| colors)
    }

    pub fn with_left_margin(self, left_margin: Coord) -> Self {
        Self { left_margin, ..self }
    }

    pub fn with_right_margin(self, right_margin: Coord) -> Self {
        Self { right_margin, ..self }
    }

    pub fn with_top_margin(self, top_margin: Coord) -> Self {
        Self { top_margin, ..self }
    }

    pub fn with_bottom_margin(self, bottom_margin: Coord) -> Self {
        Self { bottom_margin, ..self }
    }

    pub fn with_min_width(self, min_width: Coord) -> Self {
        Self { min_width, ..self }
    }

    pub fn with_max_width(self, max_width: Coord) -> Self {
        Self { max_width, ..self }
    }

    pub fn with_min_height(self, min_height: Coord) -> Self {
        Self { min_height, ..self }
    }

    pub fn with_max_height(self, max_height: Coord) -> Self {
        Self { max_height, ..self }
    }

    pub fn with_align(self, align_numer: Coord, align_denom: Coord) -> Self {
        Self { align_numer, align_denom, ..self }
    }

    pub fn left_margin(&self) -> Coord {
        self.left_margin
    }

    pub fn right_margin(&self) -> Coord {
        self.right_margin
    }

    pub fn top_margin(&self) -> Coord {
        self.top_margin
    }

    pub fn bottom_margin(&self) -> Coord {
        self.bottom_margin
    }

    pub fn min_width(&self) -> Coord {
        self.min_width
    }

    pub fn max_width(&self) -> Coord {
        self.max_width
    }

    pub fn min_height(&self) -> Coord {
        self.min_height
    }

    pub fn max_height(&self) -> Coord {
        self.max_height
    }

    pub fn align_numer(&self) -> Coord {
        self.align_numer
    }

    pub fn align_denom(&self) -> Coord {
        self.align_denom
    }

    pub fn colors(&self) -> &C {
        &self.colors
    }

    pub fn make_margin_below(&self) -> CoordPair {
        CoordPair { x: self.left_margin, y: self.top_margin }
    }

    pub fn make_margin_above(&self) -> CoordPair {
        CoordPair { x: self.right_margin, y: self.bottom_margin }
    }

    pub fn make_min_size(&self) -> CoordPair {
        CoordPair { x: self.min_width, y: self.min_height }
    }

    pub fn make_max_size(&self) -> CoordPair {
        CoordPair { x: self.max_width, y: self.max_height }
    }

    pub fn make_size(&self, canvas_size: CoordPair) -> CoordPair {
        CoordPair {
            y: canvas_size
                .y
                .saturating_sub(self.make_margin_below().y)
                .saturating_sub(self.make_margin_above().y)
                .min(self.make_max_size().y),
            x: canvas_size
                .x
                .saturating_sub(self.make_margin_below().x)
                .saturating_sub(self.make_margin_above().x)
                .min(self.make_max_size().x),
        }
    }
}
