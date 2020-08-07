pub mod string;

#[cfg(test)]
mod test;

pub use self::string::{GString, Grapheme};

use crate::math::plane::{Coord2, Nat};
use crossterm::style;
use num::{rational::Ratio, traits::FromPrimitive};
use num_derive::FromPrimitive;
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
        Self {
            fg: Color::from(BasicColor::White),
            bg: Color::from(BasicColor::Black),
        }
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
                fg: self.color.with_approx_brightness(!bg.approx_brightness()),
            },
        }
    }
}

impl Default for Foreground {
    fn default() -> Self {
        Self {
            grapheme: Grapheme::default(),
            color: Color::from(BasicColor::White),
        }
    }
}

#[cold]
#[inline(never)]
fn panic_brightness_level(found: u16) -> ! {
    panic!(
        "Brightness level must be at most {}, found {}",
        Brightness::MAX.level(),
        found
    )
}

/// The brightness of a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Brightness {
    level: u16,
}

impl Brightness {
    /// Minimum brightness (0).
    pub const MIN: Self = Self { level: 0 };
    /// Half of maximum brightness.
    pub const MAX: Self = Self { level: 391 };
    /// Maximum brightness.
    pub const HALF: Self = Self { level: Self::MAX.level() / 2 + 1 };

    /// Creates a brightness from the given brightness level.
    ///
    /// # Panics
    /// Panics if `level > MAX`.
    pub fn new(level: u16) -> Self {
        if level > Self::MAX.level() {
            panic_brightness_level(level);
        }

        Self { level }
    }

    /// Returns the level of brightness.
    pub const fn level(self) -> u16 {
        self.level
    }
}

impl Not for Brightness {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { level: Self::MAX.level() - self.level }
    }
}

/// A trait for types that can approximate their brightness.
pub trait ApproxBrightness {
    /// Approximate the brightness of the color.
    fn approx_brightness(&self) -> Brightness;
    /// Set the approximate brightness of the color.
    fn set_approx_brightness(&mut self, brightness: Brightness);

    /// Like [`set_approx_brightness`] but takes and returns `self` instead of
    /// mutating it.
    fn with_approx_brightness(mut self, brightness: Brightness) -> Self
    where
        Self: Copy,
    {
        self.set_approx_brightness(brightness);
        self
    }
}

/// A basic color used by the terminal.
#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive,
)]
pub enum BasicColor {
    /// Black.
    Black = 0,
    /// Dark red/red.
    DarkRed = 1,
    /// Dark green/green.
    DarkGreen = 2,
    /// Dark yellow/yellow.
    DarkYellow = 3,
    /// Dark blue/blue.
    DarkBlue = 4,
    /// Dark magenta/magenta.
    DarkMagenta = 5,
    /// Dark cyan/cyan.
    DarkCyan = 6,
    /// Light gray/dark white.
    LightGray = 7,
    /// Dark gray/light black.
    DarkGray = 8,
    /// Light red.
    LightRed = 9,
    /// Light green.
    LightGreen = 10,
    /// Light yellow.
    LightYellow = 11,
    /// Light blue.
    LightBlue = 12,
    /// Light magenta.
    LightMagenta = 13,
    /// Light cyan.
    LightCyan = 14,
    /// White
    White = 15,
}

impl ApproxBrightness for BasicColor {
    fn approx_brightness(&self) -> Brightness {
        match self {
            BasicColor::Black => Brightness::MIN,
            BasicColor::White => Brightness::MAX,
            BasicColor::DarkGray => Brightness::MIN,
            BasicColor::LightGray => Brightness::MAX,
            BasicColor::DarkRed => Brightness::MIN,
            BasicColor::LightRed => Brightness::MAX,
            BasicColor::DarkGreen => Brightness::MIN,
            BasicColor::LightGreen => Brightness::MAX,
            BasicColor::DarkYellow => Brightness::MIN,
            BasicColor::LightYellow => Brightness::MAX,
            BasicColor::DarkBlue => Brightness::MIN,
            BasicColor::LightBlue => Brightness::MAX,
            BasicColor::DarkMagenta => Brightness::MIN,
            BasicColor::LightMagenta => Brightness::MAX,
            BasicColor::DarkCyan => Brightness::MIN,
            BasicColor::LightCyan => Brightness::MAX,
        }
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        let self_white =
            self.approx_brightness().level() >= Brightness::HALF.level();
        let other_white = brightness.level() >= Brightness::HALF.level();

        *self = if self_white == other_white { *self } else { !*self };
    }
}

impl Not for BasicColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            BasicColor::Black => BasicColor::White,
            BasicColor::White => BasicColor::Black,
            BasicColor::DarkGray => BasicColor::LightGray,
            BasicColor::LightGray => BasicColor::DarkGray,
            BasicColor::DarkRed => BasicColor::LightRed,
            BasicColor::LightRed => BasicColor::DarkRed,
            BasicColor::DarkGreen => BasicColor::LightGreen,
            BasicColor::LightGreen => BasicColor::DarkGreen,
            BasicColor::DarkYellow => BasicColor::LightYellow,
            BasicColor::LightYellow => BasicColor::DarkYellow,
            BasicColor::DarkBlue => BasicColor::LightBlue,
            BasicColor::LightBlue => BasicColor::DarkBlue,
            BasicColor::DarkMagenta => BasicColor::LightMagenta,
            BasicColor::LightMagenta => BasicColor::DarkMagenta,
            BasicColor::DarkCyan => BasicColor::LightCyan,
            BasicColor::LightCyan => BasicColor::DarkCyan,
        }
    }
}

#[cold]
#[inline(never)]
fn panic_cmy_code(found: u8) -> ! {
    panic!(
        "CMY color components must be ast most {}, found {}.",
        CMYColor::BASE - 1,
        found
    );
}

/// A CMY (Cyan-Magenta-Yellow) color. The lower one of its component is, the
/// more it subtracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CMYColor {
    code: u8,
}

impl CMYColor {
    /// Base of CMY colors (6).
    pub const BASE: u8 = 6;

    /// Creates a new `CMYColor` given its components.
    ///
    /// # Panics
    /// Panics if any of the components is `>= 6`.
    pub fn new(cyan: u8, magenta: u8, yellow: u8) -> Self {
        if cyan >= Self::BASE || magenta >= Self::BASE || yellow >= Self::BASE {
            panic_cmy_code(cyan.max(magenta).max(yellow));
        }
        Self { code: yellow + cyan * Self::BASE + magenta * Self::BASE.pow(2) }
    }

    /// The level of cyan component.
    pub const fn cyan(self) -> u8 {
        self.code / Self::BASE % Self::BASE
    }

    /// The level of magenta component.
    pub const fn magenta(self) -> u8 {
        self.code / Self::BASE / Self::BASE % Self::BASE
    }

    /// The level of yellow component.
    pub const fn yellow(self) -> u8 {
        self.code % 6
    }

    /// The resulting code of the color.
    pub const fn code(self) -> u8 {
        self.code
    }

    /// Sets the cyan component.
    ///
    /// # Panics
    /// Panics if the components is `>= 6`.
    pub fn set_cyan(self, cyan: u8) -> Self {
        Self::new(cyan, self.magenta(), self.yellow())
    }

    /// Sets the magenta component.
    ///
    /// # Panics
    /// Panics if the components is `>= 6`.
    pub fn set_magenta(self, magenta: u8) -> Self {
        Self::new(self.cyan(), magenta, self.yellow())
    }

    /// Sets the yellow component.
    ///
    /// # Panics
    /// Panics if the components is `>= 6`.
    pub fn set_yellow(self, yellow: u8) -> Self {
        Self::new(self.cyan(), self.magenta(), yellow)
    }
}

impl Not for CMYColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(
            Self::BASE - self.cyan(),
            Self::BASE - self.magenta(),
            Self::BASE - self.yellow(),
        )
    }
}

impl ApproxBrightness for CMYColor {
    fn approx_brightness(&self) -> Brightness {
        let total = self.cyan() + self.magenta() + self.yellow();
        Brightness::new(total as u16 * 23)
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        let self_level = self.approx_brightness().level();
        let level = brightness.level() / 23;
        let ratio = Ratio::new(level, self_level);

        let cyan = Ratio::from(self.cyan() as u16) * ratio;
        let magenta = Ratio::from(self.magenta() as u16) * ratio;
        let yellow = Ratio::from(self.yellow() as u16) * ratio;

        *self = Self::new(
            cyan.to_integer() as u8,
            magenta.to_integer() as u8,
            yellow.to_integer() as u8,
        );
    }
}

#[cold]
#[inline(never)]
fn panic_gray_color(found: u8) -> ! {
    panic!(
        "Gray color must be at most {}, found {}.",
        GrayColor::MAX.lightness(),
        found
    );
}

/// A gray-scale color. Goes from white, to gray, to black.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrayColor {
    lightness: u8,
}

impl GrayColor {
    /// Minimum gray-scale lightness (0, black).
    pub const MIN: Self = Self { lightness: 0 };
    /// Half of maximum gray-scale lightness (grey).
    pub const HALF: Self = Self { lightness: 12 };
    /// Maximum gray-scale lightness (white).
    pub const MAX: Self = Self { lightness: 23 };

    /// Creates a new gray-scale color given its lightness.
    ///
    /// # Panics
    /// Panics if `lightness > MAX`.
    pub fn new(lightness: u8) -> Self {
        if lightness > Self::MAX.lightness() {
            panic_gray_color(lightness);
        }
        Self { lightness }
    }

    /// Returns the lightness of this color.
    pub const fn lightness(self) -> u8 {
        self.lightness
    }
}

impl Not for GrayColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(Self::MAX.lightness() + 1 - self.lightness)
    }
}

impl ApproxBrightness for GrayColor {
    fn approx_brightness(&self) -> Brightness {
        Brightness::new(self.lightness as u16 * 17)
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        self.lightness = (brightness.level() / 17) as u8;
    }
}

/// The kind of a color. `enum` representation of a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorKind {
    // 16 Basic colors.
    Basic(BasicColor),
    /// 216 CMY colors.
    CMY(CMYColor),
    /// 24 Gray-scale colors.
    Gray(GrayColor),
}

impl Not for ColorKind {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            ColorKind::Basic(color) => ColorKind::Basic(!color),
            ColorKind::CMY(color) => ColorKind::CMY(!color),
            ColorKind::Gray(color) => ColorKind::Gray(!color),
        }
    }
}

impl From<BasicColor> for ColorKind {
    fn from(color: BasicColor) -> Self {
        ColorKind::Basic(color)
    }
}

impl From<CMYColor> for ColorKind {
    fn from(color: CMYColor) -> Self {
        ColorKind::CMY(color)
    }
}

impl From<GrayColor> for ColorKind {
    fn from(color: GrayColor) -> Self {
        ColorKind::Gray(color)
    }
}

impl ApproxBrightness for ColorKind {
    fn approx_brightness(&self) -> Brightness {
        match self {
            ColorKind::Basic(color) => color.approx_brightness(),
            ColorKind::CMY(color) => color.approx_brightness(),
            ColorKind::Gray(color) => color.approx_brightness(),
        }
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        match self {
            ColorKind::Basic(color) => color.set_approx_brightness(brightness),
            ColorKind::CMY(color) => color.set_approx_brightness(brightness),
            ColorKind::Gray(color) => color.set_approx_brightness(brightness),
        }
    }
}

/// An 8-bit encoded color for the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color {
    code: u8,
}

impl Color {
    /// Size of basic colors.
    const BASIC_SIZE: u8 = 16;
    /// Size of basic colors + CMY colors.
    const BASIC_CMY_SIZE: u8 =
        Self::BASIC_SIZE + CMYColor::BASE * CMYColor::BASE * CMYColor::BASE;

    /// Creates a color that is basic.
    pub const fn basic(color: BasicColor) -> Self {
        Self { code: color as u8 }
    }

    /// Creates a color that is CMY.
    pub const fn cmy(color: CMYColor) -> Self {
        Self { code: color.code() + Self::BASIC_SIZE }
    }

    /// Creates a color that is gray-scale.
    pub const fn gray(color: GrayColor) -> Self {
        Self { code: color.lightness() + Self::BASIC_CMY_SIZE }
    }

    /// Returns the color code.
    pub const fn code(self) -> u8 {
        self.code
    }

    /// Converts to en `enum` representation.
    pub fn kind(self) -> ColorKind {
        if self.code < 16 {
            ColorKind::Basic(BasicColor::from_u8(self.code).unwrap())
        } else if self.code < Self::BASIC_CMY_SIZE {
            ColorKind::CMY(CMYColor { code: self.code - Self::BASIC_SIZE })
        } else {
            ColorKind::Gray(GrayColor {
                lightness: self.code - Self::BASIC_CMY_SIZE,
            })
        }
    }

    pub(crate) fn translate(self) -> style::Color {
        style::Color::AnsiValue(self.code())
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::from(!self.kind())
    }
}

impl From<BasicColor> for Color {
    fn from(color: BasicColor) -> Self {
        Self::basic(color)
    }
}

impl From<CMYColor> for Color {
    fn from(color: CMYColor) -> Self {
        Self::cmy(color)
    }
}

impl From<GrayColor> for Color {
    fn from(color: GrayColor) -> Self {
        Self::gray(color)
    }
}

impl From<ColorKind> for Color {
    fn from(kind: ColorKind) -> Self {
        match kind {
            ColorKind::Basic(color) => Self::from(color),
            ColorKind::CMY(color) => Self::from(color),
            ColorKind::Gray(color) => Self::from(color),
        }
    }
}

impl ApproxBrightness for Color {
    fn approx_brightness(&self) -> Brightness {
        self.kind().approx_brightness()
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        *self = Self::from(self.kind().with_approx_brightness(brightness));
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
