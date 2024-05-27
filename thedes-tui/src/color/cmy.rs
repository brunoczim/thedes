use std::{fmt, ops::Not};

use thiserror::Error;

use super::{
    brightness::{Channel, ChannelVector},
    ApproxBrightness,
    Brightness,
};

/// Error returned when a [`CmyColor`](crate::color::CmyColor) is attempted to
/// be created with invalid channels.
#[derive(Debug, Error)]
#[error("Bad CMY color, cyan={cyan}, magenta={magenta}, yellow={yellow}")]
pub struct BadCmyColor {
    /// The cyan channel given to [`CmyColor`](crate::color::CmyColor).
    pub cyan: u8,
    /// The magenta channel given to [`CmyColor`](crate::color::CmyColor).
    pub magenta: u8,
    /// The yellow channel given to [`CmyColor`](crate::color::CmyColor).
    pub yellow: u8,
}

/// Error returned when a [`CmyColor`](crate::color::CmyColor) is attempted
/// to be created with an invalid code.
#[derive(Debug, Error)]
#[error("Bad CMY color code {code}")]
pub struct BadCmyColorCode {
    /// The code given to [`CmyColor`](crate::color::CmyColor).
    pub code: u8,
}

/// A CMY (Cyan-Magenta-Yellow) color. The lower one of its component is, the
/// more it subtracts.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CmyColor {
    /// `(0 .. 216)` Color code.
    code: u8,
}

impl CmyColor {
    /// Base of CMY colors (6).
    pub const BASE: u8 = 6;

    /// Number of channels.
    pub const CHANNELS: usize = 3;

    /// Creates a new `CmyColor` given its components. Returns an error if any
    /// of the components is `>= `[`Self::BASE`].
    pub fn try_new(
        cyan: u8,
        magenta: u8,
        yellow: u8,
    ) -> Result<Self, BadCmyColor> {
        if cyan >= Self::BASE || magenta >= Self::BASE || yellow >= Self::BASE {
            Err(BadCmyColor { cyan, magenta, yellow })
        } else {
            Ok(Self {
                code: cyan * Self::BASE.pow(2) + magenta * Self::BASE + yellow,
            })
        }
    }

    /// Creates a new `CmyColor` given its components.
    ///
    /// # Panics
    /// Panics if any of the components is `>= `[`Self::BASE`].
    pub fn new(cyan: u8, magenta: u8, yellow: u8) -> Self {
        Self::try_new(cyan, magenta, yellow).expect("CMY color should be valid")
    }

    pub fn from_code(code: u8) -> Result<Self, BadCmyColorCode> {
        if code >= Self::BASE.pow(Self::CHANNELS as u32) {
            Err(BadCmyColorCode { code })
        } else {
            Ok(Self { code })
        }
    }

    /// The level of cyan component.
    pub const fn cyan(self) -> u8 {
        self.code / Self::BASE / Self::BASE % Self::BASE
    }

    /// The level of magenta component.
    pub const fn magenta(self) -> u8 {
        self.code / Self::BASE % Self::BASE
    }

    /// The level of yellow component.
    pub const fn yellow(self) -> u8 {
        self.code % Self::BASE
    }

    /// The resulting code of the color.
    pub const fn code(self) -> u8 {
        self.code
    }

    /// Sets the cyan component.
    ///
    /// # Panics
    /// Panics if the component is `>= `[`Self::BASE`].
    pub fn set_cyan(self, cyan: u8) -> Self {
        Self::new(cyan, self.magenta(), self.yellow())
    }

    /// Sets the magenta component.
    ///
    /// # Panics
    /// Panics if the component is `>= `[`Self::BASE`].
    pub fn set_magenta(self, magenta: u8) -> Self {
        Self::new(self.cyan(), magenta, self.yellow())
    }

    /// Sets the yellow component.
    ///
    /// # Panics
    /// Panics if the component is `>= `[`Self::BASE`].
    pub fn set_yellow(self, yellow: u8) -> Self {
        Self::new(self.cyan(), self.magenta(), yellow)
    }

    /// Creates a CMY color from the given channels.
    fn from_channels(channels: [Channel; Self::CHANNELS]) -> Self {
        Self::new(channels[0].value(), channels[1].value(), channels[2].value())
    }

    /// Returns a CMY color's channels.
    fn channels(self) -> [Channel; Self::CHANNELS] {
        [
            Channel::new(self.cyan(), 30),
            Channel::new(self.magenta(), 59),
            Channel::new(self.yellow(), 11),
        ]
    }
}

impl fmt::Debug for CmyColor {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmtr.debug_struct("CmyColor")
            .field("cyan", &self.cyan())
            .field("magenta", &self.magenta())
            .field("yellow", &self.yellow())
            .finish()
    }
}

impl Not for CmyColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(
            Self::BASE - self.cyan(),
            Self::BASE - self.magenta(),
            Self::BASE - self.yellow(),
        )
    }
}

impl ApproxBrightness for CmyColor {
    fn approx_brightness(&self) -> Brightness {
        let mut channels = self.channels();
        let vector = ChannelVector::new(&mut channels, Self::BASE - 1);
        vector.approx_brightness()
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        let mut channels = self.channels();
        let mut vector = ChannelVector::new(&mut channels, Self::BASE - 1);
        vector.set_approx_brightness(brightness);
        *self = Self::from_channels(channels);
    }
}
