use crate::backend::{Backend, Render};
use std::{
    io,
    ops::{Index, IndexMut},
};

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A kind of horizontal orientation, position, and/or alignment.
pub enum OrientX {
    /// Orientation to the left side.
    Left,
    /// Orientation to the horizontal center.
    Mid,
    /// Orientation to the right side.
    Right,
}

impl Default for OrientX {
    fn default() -> Self {
        OrientX::Mid
    }
}

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A kind of vertical orientation, position, and/or alignment.
pub enum OrientY {
    /// Orientation to the top.
    Top,
    /// Orientation to the vertical center.
    Mid,
    /// Orientation to the bottom.
    Bottom,
}

impl Default for OrientY {
    fn default() -> Self {
        OrientY::Mid
    }
}

/// Orientation tuples, i.e. combinations of vertical and horizontal
/// orientations.
pub mod orient {
    use super::{OrientX, OrientY};

    /// Orientation to the true left side.
    pub const LEFT: (OrientX, OrientY) = (OrientX::Left, OrientY::Mid);
    /// Orientation to the true right side.
    pub const RIGHT: (OrientX, OrientY) = (OrientX::Right, OrientY::Mid);
    /// Orientation to the true top.
    pub const TOP: (OrientX, OrientY) = (OrientX::Mid, OrientY::Top);
    /// Orientation to the true bottom.
    pub const BOTTOM: (OrientX, OrientY) = (OrientX::Mid, OrientY::Bottom);
    /// Orientation to both the top and left side.
    pub const TOP_LEFT: (OrientX, OrientY) = (OrientX::Left, OrientY::Top);
    /// Orientation to both the top and right side.
    pub const TOP_RIGHT: (OrientX, OrientY) = (OrientX::Right, OrientY::Top);
    /// Orientation to both the bottom and left side.
    pub const BOT_LEFT: (OrientX, OrientY) = (OrientX::Left, OrientY::Bottom);
    /// Orientation to both the bottom and right side.
    pub const BOT_RIGHT: (OrientX, OrientY) = (OrientX::Right, OrientY::Bottom);
    /// Orientation to the true center.
    pub const TRUE_MID: (OrientX, OrientY) = (OrientX::Mid, OrientY::Mid);
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

/// A cell occuping a single char on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    /// The char ilustrating this cell.
    pub ch: char,
    /// The foreground color of this cell.
    pub fg: Color,
    /// The background color of this cell.
    pub bg: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self { ch: ' ', fg: Color::White, bg: Color::Black }
    }
}

impl Render for Cell {
    fn render<B>(&self, backend: &mut B) -> io::Result<()>
    where
        B: Backend,
    {
        backend.setfg(self.fg)?;
        backend.setbg(self.bg)?;
        write!(backend, "{}", self.ch)
    }
}

#[derive(Debug, Default, Clone)]
pub struct TileSet {
    tiles: Box<[Cell]>,
}
