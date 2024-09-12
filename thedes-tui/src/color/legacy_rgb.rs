use std::{fmt, ops::Not};

use thiserror::Error;

use super::{
    brightness::{Channel, ChannelVector},
    ApproxBrightness,
    Brightness,
};

#[derive(Debug, Error)]
#[error("Bad legacy RGB color, red={red}, green={green}, blue={blue}")]
pub struct BadLegacyColor {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
}

#[derive(Debug, Error)]
#[error("Bad CMY color code {code}")]
pub struct BadLegacyRgbColorCode {
    pub code: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LegacyRgb {
    /// `(0 .. 216)` Color code.
    code: u8,
}

impl LegacyRgb {
    pub const BASE: u8 = 6;

    pub const CHANNELS: usize = 3;

    pub fn try_new(
        red: u8,
        green: u8,
        blue: u8,
    ) -> Result<Self, BadLegacyColor> {
        if red >= Self::BASE || green >= Self::BASE || blue >= Self::BASE {
            Err(BadLegacyColor { red, green, blue })
        } else {
            Ok(Self {
                code: red * Self::BASE.pow(2) + green * Self::BASE + blue,
            })
        }
    }

    /// Creates a new `LegacyRgbColor` given its components.
    ///
    /// # Panics
    /// Panics if any of the components is `>= `[`Self::BASE`].
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self::try_new(red, green, blue)
            .expect("Legacy RGB color should be valid")
    }

    pub fn from_code(code: u8) -> Result<Self, BadLegacyRgbColorCode> {
        if code >= Self::BASE.pow(Self::CHANNELS as u32) {
            Err(BadLegacyRgbColorCode { code })
        } else {
            Ok(Self { code })
        }
    }

    pub const fn red(self) -> u8 {
        self.code / Self::BASE / Self::BASE % Self::BASE
    }

    pub const fn green(self) -> u8 {
        self.code / Self::BASE % Self::BASE
    }

    pub const fn blue(self) -> u8 {
        self.code % Self::BASE
    }

    pub const fn code(self) -> u8 {
        self.code
    }

    pub fn set_red(self, red: u8) -> Self {
        Self::new(red, self.green(), self.blue())
    }

    pub fn set_green(self, green: u8) -> Self {
        Self::new(self.red(), green, self.blue())
    }

    pub fn set_blue(self, blue: u8) -> Self {
        Self::new(self.red(), self.green(), blue)
    }

    fn from_channels(channels: [Channel; Self::CHANNELS]) -> Self {
        Self::new(channels[0].value(), channels[1].value(), channels[2].value())
    }

    fn channels(self) -> [Channel; Self::CHANNELS] {
        [
            Channel::new(self.red(), 299),
            Channel::new(self.green(), 587),
            Channel::new(self.blue(), 114),
        ]
    }
}

impl fmt::Debug for LegacyRgb {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmtr.debug_struct("LegacyRgbColor")
            .field("red", &self.red())
            .field("green", &self.green())
            .field("blue", &self.blue())
            .finish()
    }
}

impl Not for LegacyRgb {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(
            Self::BASE - self.red(),
            Self::BASE - self.green(),
            Self::BASE - self.blue(),
        )
    }
}

impl ApproxBrightness for LegacyRgb {
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
