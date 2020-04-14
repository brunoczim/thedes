pub mod string;

#[cfg(test)]
mod test;

pub use self::string::{GString, Grapheme};

use crate::coord::{Coord2, Nat};
use crossterm::style;
use std::ops::Not;

/// A screen's tile content. Includes a grapheme and colors.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Tile {
    /// Grapheme shown in this tile.
    pub grapheme: Grapheme,
    /// The foreground-background pair of colors.
    pub colors: Color2,
}

impl Tile {
    /// Converts this tile into a foreground only tile.
    pub fn fg(self) -> Foreground {
        Foreground { grapheme: self.grapheme, color: self.colors.fg }
    }
}

/// A pair of colors representing foreground and background colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color2 {
    /// The background color.
    pub fg: Color,
    /// The foreground color.
    pub bg: Color,
}

impl Default for Color2 {
    fn default() -> Self {
        Self { fg: Color::White, bg: Color::Black }
    }
}

impl Not for Color2 {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { fg: !self.fg, bg: !self.bg }
    }
}

/// The foreground of a tile.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Foreground {
    /// The shown grapheme.
    pub grapheme: Grapheme,
    /// The color of the grapheme.
    pub color: Color,
}

impl Foreground {
    /// Makes a tile with contrasting color relative to the given background.
    pub fn make_tile(self, bg: Color) -> Tile {
        Tile {
            grapheme: self.grapheme,
            colors: Color2 {
                bg,
                fg: self.color.set_brightness(!bg.brightness()),
            },
        }
    }
}

impl Default for Foreground {
    fn default() -> Self {
        Self { grapheme: Grapheme::default(), color: Color::White }
    }
}

/// Brightness of a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Brightness {
    /// This is a light color.
    Light,
    /// This is a dark color.
    Dark,
}

impl Not for Brightness {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Brightness::Light => Brightness::Dark,
            Brightness::Dark => Brightness::Light,
        }
    }
}

/// A color used by the terminal. Either dark or light.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    /// Black
    Black,
    /// White
    White,
    /// Dark LightGrey
    DarkGrey,
    /// Light LightGrey
    LightGrey,
    /// Dark LightRed
    DarkRed,
    /// Light LightRed
    LightRed,
    /// Dark LightGreen
    DarkGreen,
    /// Light LightGreen
    LightGreen,
    /// Dark LightYellow
    DarkYellow,
    /// Light LightYellow
    LightYellow,
    /// Dark LightBlue
    DarkBlue,
    /// Light LightBlue
    LightBlue,
    /// Dark LightMagenta
    DarkMagenta,
    /// Light LightMagenta
    LightMagenta,
    /// Dark LightCyan
    DarkCyan,
    /// Light LightCyan
    LightCyan,
}

impl Color {
    /// Returns the brightness of the color.
    pub fn brightness(self) -> Brightness {
        match self {
            Color::Black => Brightness::Dark,
            Color::White => Brightness::Light,
            Color::DarkGrey => Brightness::Dark,
            Color::LightGrey => Brightness::Light,
            Color::DarkRed => Brightness::Dark,
            Color::LightRed => Brightness::Light,
            Color::DarkGreen => Brightness::Dark,
            Color::LightGreen => Brightness::Light,
            Color::DarkYellow => Brightness::Dark,
            Color::LightYellow => Brightness::Light,
            Color::DarkBlue => Brightness::Dark,
            Color::LightBlue => Brightness::Light,
            Color::DarkMagenta => Brightness::Dark,
            Color::LightMagenta => Brightness::Light,
            Color::DarkCyan => Brightness::Dark,
            Color::LightCyan => Brightness::Light,
        }
    }

    /// Sets the brightness of the current color to match the given brightness.
    pub fn set_brightness(self, brightness: Brightness) -> Self {
        if self.brightness() == brightness {
            self
        } else {
            !self
        }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
            Color::DarkGrey => Color::LightGrey,
            Color::LightGrey => Color::DarkGrey,
            Color::DarkRed => Color::LightRed,
            Color::LightRed => Color::DarkRed,
            Color::DarkGreen => Color::LightGreen,
            Color::LightGreen => Color::DarkGreen,
            Color::DarkYellow => Color::LightYellow,
            Color::LightYellow => Color::DarkYellow,
            Color::DarkBlue => Color::LightBlue,
            Color::LightBlue => Color::DarkBlue,
            Color::DarkMagenta => Color::LightMagenta,
            Color::LightMagenta => Color::DarkMagenta,
            Color::DarkCyan => Color::LightCyan,
            Color::LightCyan => Color::DarkCyan,
        }
    }
}

pub(crate) fn translate_color(color: Color) -> style::Color {
    match color {
        Color::White => style::Color::White,
        Color::Black => style::Color::Black,
        Color::DarkGrey => style::Color::DarkGrey,
        Color::LightGrey => style::Color::Grey,
        Color::DarkRed => style::Color::DarkRed,
        Color::LightRed => style::Color::Red,
        Color::DarkGreen => style::Color::DarkGreen,
        Color::LightGreen => style::Color::Green,
        Color::DarkYellow => style::Color::DarkYellow,
        Color::LightYellow => style::Color::Yellow,
        Color::DarkBlue => style::Color::DarkBlue,
        Color::LightBlue => style::Color::Blue,
        Color::DarkMagenta => style::Color::DarkMagenta,
        Color::LightMagenta => style::Color::Magenta,
        Color::DarkCyan => style::Color::DarkCyan,
        Color::LightCyan => style::Color::Cyan,
    }
}

/// Alignment and margin settings for texts.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// Left margin.
    pub left_margin: Nat,
    /// Right margin.
    pub right_margin: Nat,
    /// Top margin.
    pub top_margin: Nat,
    /// Bottom margin.
    pub bottom_margin: Nat,
    /// Minimum width.
    pub min_width: Nat,
    /// Maximum width.
    pub max_width: Nat,
    /// Minimum height.
    pub min_height: Nat,
    /// Maximum height.
    pub max_height: Nat,
    /// Alignment numerator.
    pub num: Nat,
    /// Alignment denominator.
    pub den: Nat,
    /// Foreground-background color pair.
    pub colors: Color2,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            left_margin: 0,
            right_margin: 0,
            top_margin: 0,
            bottom_margin: 0,
            min_width: 0,
            max_width: Nat::max_value(),
            min_height: 0,
            max_height: Nat::max_value(),
            num: 0,
            den: 1,
            colors: Color2::default(),
        }
    }
}

impl Style {
    /// Default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets left margin.
    pub fn left_margin(self, left_margin: Nat) -> Self {
        Self { left_margin, ..self }
    }

    /// Sets right margin.
    pub fn right_margin(self, right_margin: Nat) -> Self {
        Self { right_margin, ..self }
    }

    /// Sets top margin.
    pub fn top_margin(self, top_margin: Nat) -> Self {
        Self { top_margin, ..self }
    }

    /// Sets bottom margin.
    pub fn bottom_margin(self, bottom_margin: Nat) -> Self {
        Self { bottom_margin, ..self }
    }

    /// Sets minimum width.
    pub fn min_width(self, min_width: Nat) -> Self {
        Self { min_width, ..self }
    }

    /// Sets maximum width.
    pub fn max_width(self, max_width: Nat) -> Self {
        Self { max_width, ..self }
    }

    /// Sets minimum height.
    pub fn min_height(self, min_height: Nat) -> Self {
        Self { min_height, ..self }
    }

    /// Sets maximum height.
    pub fn max_height(self, max_height: Nat) -> Self {
        Self { max_height, ..self }
    }

    /// Sets alignment. Numerator and denominator are used such that
    /// `line\[index\] * num / den == screen\[index\]`
    pub fn align(self, num: Nat, den: Nat) -> Self {
        Self { num, den, ..self }
    }

    /// Sets foreground and background colors.
    pub fn colors(self, colors: Color2) -> Self {
        Self { colors, ..self }
    }

    /// Makes a coordinate pair that contains the margin dimensions that are
    /// "less".
    pub fn make_margin_below(self) -> Coord2<Nat> {
        Coord2 { x: self.left_margin, y: self.top_margin }
    }

    /// Makes a coordinate pair that contains the margin dimensions that are
    /// "greater".
    pub fn make_margin_above(self) -> Coord2<Nat> {
        Coord2 { x: self.right_margin, y: self.bottom_margin }
    }

    /// Makes a coordinate pair that contains the minima sizes.
    pub fn make_min_size(self) -> Coord2<Nat> {
        Coord2 { x: self.min_width, y: self.min_height }
    }

    /// Makes a coordinate pair that contains the maxima sizes.
    pub fn make_max_size(self) -> Coord2<Nat> {
        Coord2 { x: self.max_width, y: self.max_height }
    }

    /// Makes a coordinate pair that contains the actual sizes.
    pub fn make_size(self, screen_size: Coord2<Nat>) -> Coord2<Nat> {
        Coord2::from_axes(|axis| {
            screen_size[axis]
                .saturating_sub(self.make_margin_below()[axis])
                .saturating_sub(self.make_margin_above()[axis])
                .min(self.make_max_size()[axis])
        })
    }
}
