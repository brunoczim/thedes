//! This module provides 8-bit color utilies.

use crate::color::{ApproxBrightness, BasicColor, Brightness};
use crossterm::style::Color as CrosstermColor;
use std::{convert::TryFrom, fmt, ops::Not};

use super::{cmy::CmyColor, gray::GrayColor};

/// The kind of a color. `enum` representation of an 8-bit color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color8BitKind {
    /// 16 Basic colors.
    Basic(BasicColor),
    /// 216 CMY colors.
    Cmy(CmyColor),
    /// 24 Gray-scale colors.
    Gray(GrayColor),
}

impl Not for Color8BitKind {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color8BitKind::Basic(color) => Color8BitKind::Basic(!color),
            Color8BitKind::Cmy(color) => Color8BitKind::Cmy(!color),
            Color8BitKind::Gray(color) => Color8BitKind::Gray(!color),
        }
    }
}

impl From<BasicColor> for Color8BitKind {
    fn from(color: BasicColor) -> Self {
        Color8BitKind::Basic(color)
    }
}

impl From<CmyColor> for Color8BitKind {
    fn from(color: CmyColor) -> Self {
        Color8BitKind::Cmy(color)
    }
}

impl From<GrayColor> for Color8BitKind {
    fn from(color: GrayColor) -> Self {
        Color8BitKind::Gray(color)
    }
}

impl From<Color8Bit> for Color8BitKind {
    fn from(color: Color8Bit) -> Self {
        color.kind()
    }
}

impl ApproxBrightness for Color8BitKind {
    fn approx_brightness(&self) -> Brightness {
        match self {
            Color8BitKind::Basic(color) => color.approx_brightness(),
            Color8BitKind::Cmy(color) => color.approx_brightness(),
            Color8BitKind::Gray(color) => color.approx_brightness(),
        }
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        match self {
            Color8BitKind::Basic(color) => {
                color.set_approx_brightness(brightness)
            },
            Color8BitKind::Cmy(color) => {
                color.set_approx_brightness(brightness)
            },
            Color8BitKind::Gray(color) => {
                color.set_approx_brightness(brightness)
            },
        }
    }
}

/// An 8-bit encoded color for the terminal.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color8Bit {
    code: u8,
}

impl Color8Bit {
    /// Size of basic colors.
    const BASIC_SIZE: u8 = 16;
    /// Size of basic colors + CMY colors.
    const BASIC_CMY_SIZE: u8 =
        Self::BASIC_SIZE + CmyColor::BASE * CmyColor::BASE * CmyColor::BASE;

    /// Creates a color that is basic.
    pub const fn basic(color: BasicColor) -> Self {
        Self { code: color as u8 }
    }

    /// Creates a color that is CMY.
    pub const fn cmy(color: CmyColor) -> Self {
        Self { code: color.code() + Self::BASIC_SIZE }
    }

    /// Creates a color that is gray-scale.
    pub const fn gray(color: GrayColor) -> Self {
        Self { code: color.brightness() + Self::BASIC_CMY_SIZE }
    }

    /// Returns the color code.
    pub const fn code(self) -> u8 {
        self.code
    }

    /// Converts to en `enum` representation.
    pub fn kind(self) -> Color8BitKind {
        if self.code < 16 {
            Color8BitKind::Basic(
                BasicColor::try_from(self.code).expect(
                    "Basic color code of 8-bit color should be consistent",
                ),
            )
        } else if self.code < Self::BASIC_CMY_SIZE {
            Color8BitKind::Cmy(
                CmyColor::from_code(self.code - Self::BASIC_SIZE)
                    .expect("CMY color of 8-bit color should be consistent"),
            )
        } else {
            Color8BitKind::Gray(
                GrayColor::try_new(self.code - Self::BASIC_CMY_SIZE).expect(
                    "Gray color code of 8-bit color should be consistent",
                ),
            )
        }
    }

    /// Translates this color to a crossterm color.
    pub(crate) fn to_crossterm(self) -> CrosstermColor {
        CrosstermColor::AnsiValue(self.code())
    }
}

impl fmt::Debug for Color8Bit {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmtr.debug_struct("Color8Bit").field("kind", &self.kind()).finish()
    }
}

impl Not for Color8Bit {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::from(!self.kind())
    }
}

impl From<BasicColor> for Color8Bit {
    fn from(color: BasicColor) -> Self {
        Self::basic(color)
    }
}

impl From<CmyColor> for Color8Bit {
    fn from(color: CmyColor) -> Self {
        Self::cmy(color)
    }
}

impl From<GrayColor> for Color8Bit {
    fn from(color: GrayColor) -> Self {
        Self::gray(color)
    }
}

impl From<Color8BitKind> for Color8Bit {
    fn from(kind: Color8BitKind) -> Self {
        match kind {
            Color8BitKind::Basic(color) => Self::from(color),
            Color8BitKind::Cmy(color) => Self::from(color),
            Color8BitKind::Gray(color) => Self::from(color),
        }
    }
}

impl ApproxBrightness for Color8Bit {
    fn approx_brightness(&self) -> Brightness {
        self.kind().approx_brightness()
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        *self = Self::from(self.kind().with_approx_brightness(brightness));
    }
}
