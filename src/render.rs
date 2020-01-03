use crate::orient::{Coord, Coord2D};

/// Minimum size supported for screen.
pub const MIN_SCREEN: Coord2D = Coord2D { x: 80, y: 24 };

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
