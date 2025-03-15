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

mod brightness;
mod basic;

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
}

impl From<BasicColor> for Color {
    fn from(color: BasicColor) -> Self {
        Color::Basic(color)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::from(BasicColor::default())
    }
}
