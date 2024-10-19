use std::ops::Not;

use thiserror::Error;

use super::{ApproxBrightness, Brightness};

/// Error returned when a [`GrayColor`](crate::color::GrayColor) is attempted to
/// be created with an invalid brightness.
#[derive(Debug, Error)]
#[error("Bad gray color brightness {brightness}")]
pub struct BadGrayColor {
    /// The code given to [`GrayColor`](crate::color::GrayColor).
    pub brightness: u8,
}

/// A gray-scale color. Goes from white, to gray, to black.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrayColor {
    /// Level of white.
    brightness: u8,
}

impl GrayColor {
    /// Minimum gray-scale brightness (0, black).
    pub const MIN: Self = Self { brightness: 0 };
    /// Half of maximum gray-scale brightness (gray).
    pub const HALF: Self = Self { brightness: 12 };
    /// Maximum gray-scale brightness (white).
    pub const MAX: Self = Self { brightness: 23 };

    /// Creates a new gray-scale color given its brightness. Returns an error if
    /// `brightness > MAX`.
    pub fn try_new(brightness: u8) -> Result<Self, BadGrayColor> {
        if brightness > Self::MAX.brightness() {
            Err(BadGrayColor { brightness })
        } else {
            Ok(Self { brightness })
        }
    }

    /// Creates a new gray-scale color given its brightness.
    ///
    /// # Panics
    /// Panics if `brightness > MAX`.
    pub fn new(brightness: u8) -> Self {
        Self::try_new(brightness).expect("Bad gray color")
    }

    /// Returns the brightness of this color.
    pub const fn brightness(self) -> u8 {
        self.brightness
    }
}

impl Not for GrayColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(Self::MAX.brightness() + 1 - self.brightness)
    }
}

impl ApproxBrightness for GrayColor {
    fn approx_brightness(&self) -> Brightness {
        let brightness = Brightness { level: u16::from(self.brightness) };
        brightness.spread(u16::from(Self::MAX.brightness))
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        let compressed =
            brightness.compress_raw(u16::from(Self::MAX.brightness));
        let res = u8::try_from(compressed.level);
        self.brightness = res.expect("Color brightness bug");
    }
}
