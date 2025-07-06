pub use basic::{
    BadBassicColorCode,
    BasicColor,
    BasicColorParts,
    BasicColorVariant,
};
pub use brightness::{
    ApproxBrightness,
    Brightness,
    BrightnessError,
    BrightnessLevel,
    MutableApproxBrightness,
};
use grayscale::Grayscale;
pub use legacy_rgb::{
    BadLegacyLevel,
    BadLegacyRgbCode,
    LegacyLevel,
    LegacyRgb,
};
pub use rgb::Rgb;

mod brightness;
mod channel_vector;
mod basic;
mod legacy_rgb;
mod rgb;
mod grayscale;

pub(crate) mod native_ext;

pub mod mutation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorPair {
    pub background: Color,
    pub foreground: Color,
}

impl Default for ColorPair {
    fn default() -> Self {
        Self {
            background: BasicColor::Black.into(),
            foreground: BasicColor::White.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    Basic(BasicColor),
    LegacyRgb(LegacyRgb),
    Rgb(Rgb),
    Grayscale(Grayscale),
}

impl From<BasicColor> for Color {
    fn from(color: BasicColor) -> Self {
        Color::Basic(color)
    }
}

impl From<LegacyRgb> for Color {
    fn from(color: LegacyRgb) -> Self {
        Color::LegacyRgb(color)
    }
}

impl From<Rgb> for Color {
    fn from(color: Rgb) -> Self {
        Color::Rgb(color)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::from(BasicColor::default())
    }
}

impl ApproxBrightness for Color {
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        match self {
            Self::Basic(color) => color.approx_brightness(),
            Self::LegacyRgb(color) => color.approx_brightness(),
            Self::Rgb(color) => color.approx_brightness(),
            Self::Grayscale(color) => color.approx_brightness(),
        }
    }
}

impl MutableApproxBrightness for Color {
    fn set_approx_brightness(
        &mut self,
        brightness: Brightness,
    ) -> Result<(), BrightnessError> {
        match self {
            Self::Basic(color) => color.set_approx_brightness(brightness),
            Self::LegacyRgb(color) => color.set_approx_brightness(brightness),
            Self::Rgb(color) => color.set_approx_brightness(brightness),
            Self::Grayscale(color) => color.set_approx_brightness(brightness),
        }
    }
}
