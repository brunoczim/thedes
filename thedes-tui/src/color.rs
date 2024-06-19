//! This module provides colors that are usable with the terminal handle
//! implemented by this library.

#[cfg(test)]
mod test;

mod brightness;
mod basic;
mod legacy_rgb;
mod gray;
mod eight_bit;
mod rgb;
mod pair;

use crossterm::style::Color as CrosstermColor;
use std::ops::Not;

pub use self::{
    basic::BasicColor,
    brightness::{ApproxBrightness, Brightness},
    eight_bit::{Color8BitKind, EightBitColor},
    gray::GrayColor,
    legacy_rgb::LegacyRgb,
    pair::{
        AdaptBgToFg,
        AdaptFgToBg,
        ColorPair,
        ContrastBgWithFg,
        ContrastFgWithBg,
        Mutation,
        UpdateBg,
        UpdateFg,
    },
    rgb::RgbColor,
};

/// A color usable in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    /// A basic color. Totals 16 colors. By far, the most portable color set.
    Basic(BasicColor),
    /// ANSI 8-bit color. Totals 256 colors: 16 basic colors (likely the same
    /// as `Color::Basic`), 216 legacy RGB Colors and 24 gray-scale colors. Not
    /// as portable as `Color::Basic`, but still portable (it's ANSI).
    EightBit(EightBitColor),
    /// Common 24 bit RGB colors (Red-Green-Blue), each channel a byte. Not
    /// very portable, but some terminals do implement it.
    Rgb(RgbColor),
}

impl Color {
    /// Translates this color to a crossterm color.
    pub(crate) fn to_crossterm(self) -> CrosstermColor {
        match self {
            Color::Basic(color) => color.to_crossterm(),
            Color::EightBit(color) => color.to_crossterm(),
            Color::Rgb(color) => color.to_crossterm(),
        }
    }
}

impl ApproxBrightness for Color {
    fn approx_brightness(&self) -> Brightness {
        match self {
            Color::Basic(color) => color.approx_brightness(),
            Color::EightBit(color) => color.approx_brightness(),
            Color::Rgb(color) => color.approx_brightness(),
        }
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        match self {
            Color::Basic(color) => color.set_approx_brightness(brightness),
            Color::EightBit(color) => color.set_approx_brightness(brightness),
            Color::Rgb(color) => color.set_approx_brightness(brightness),
        }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::Basic(color) => Color::Basic(!color),
            Color::EightBit(color) => Color::EightBit(!color),
            Color::Rgb(color) => Color::Rgb(!color),
        }
    }
}

impl From<BasicColor> for Color {
    fn from(color: BasicColor) -> Self {
        Color::Basic(color)
    }
}

impl From<EightBitColor> for Color {
    fn from(color: EightBitColor) -> Self {
        Color::EightBit(color)
    }
}

impl From<Color8BitKind> for Color {
    fn from(kind: Color8BitKind) -> Self {
        Color::from(EightBitColor::from(kind))
    }
}

impl From<LegacyRgb> for Color {
    fn from(color: LegacyRgb) -> Self {
        Color::from(EightBitColor::from(color))
    }
}

impl From<GrayColor> for Color {
    fn from(color: GrayColor) -> Self {
        Color::from(EightBitColor::from(color))
    }
}

impl From<RgbColor> for Color {
    fn from(color: RgbColor) -> Self {
        Color::Rgb(color)
    }
}
