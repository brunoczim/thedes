//! This module provides an RGB color utilites.

use crate::color::{
    brightness::{Channel, ChannelVector},
    ApproxBrightness,
    Brightness,
};
use crossterm::style::Color as CrosstermColor;
use std::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
    pub const CHANNELS: usize = 3;

    /// Translates this color to a crossterm color.
    pub(crate) fn to_crossterm(self) -> CrosstermColor {
        CrosstermColor::Rgb { r: self.red, g: self.green, b: self.blue }
    }

    fn from_channels(channels: [Channel; Self::CHANNELS]) -> Self {
        Self {
            red: channels[0].value(),
            green: channels[1].value(),
            blue: channels[2].value(),
        }
    }

    fn channels(self) -> [Channel; Self::CHANNELS] {
        [
            Channel::new(self.red, 30),
            Channel::new(self.green, 59),
            Channel::new(self.blue, 11),
        ]
    }
}

impl ApproxBrightness for RgbColor {
    fn approx_brightness(&self) -> Brightness {
        let mut channels = self.channels();
        let vector = ChannelVector::new(&mut channels, u8::max_value());
        vector.approx_brightness()
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        let mut channels = self.channels();
        let mut vector = ChannelVector::new(&mut channels, u8::max_value());
        vector.set_approx_brightness(brightness);
        *self = Self::from_channels(channels);
    }
}

impl Not for RgbColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        let max = u8::max_value();
        Self {
            red: max - self.red,
            green: max - self.green,
            blue: max - self.blue,
        }
    }
}
