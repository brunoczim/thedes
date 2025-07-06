use thiserror::Error;

use super::{
    ApproxBrightness,
    Brightness,
    BrightnessError,
    MutableApproxBrightness,
};

#[derive(Debug, Clone, Copy, Error)]
#[error("Invalid grayscale level {0}")]
pub struct InvalidGrayscaleLevel(pub u8);

#[derive(Debug, Clone, Copy, Error)]
#[error("Invalid grayscale code {0}")]
pub struct InvalidGrayscaleCode(pub u8);

const CODE_OFFSET: u8 = 255 - Grayscale::MAX.level();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Grayscale(u8);

impl Grayscale {
    pub const MAX: Self = Self(23);

    pub fn new(level: u8) -> Result<Self, InvalidGrayscaleLevel> {
        if level > Self::MAX.level() {
            Err(InvalidGrayscaleLevel(level))?
        }
        Ok(Self(level))
    }

    pub const fn level(self) -> u8 {
        self.0
    }

    pub fn code(self) -> u8 {
        self.level() + CODE_OFFSET
    }
}

impl TryFrom<u8> for Grayscale {
    type Error = InvalidGrayscaleCode;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        let Some(level) = code.checked_sub(CODE_OFFSET) else {
            Err(InvalidGrayscaleCode(code))?
        };
        Self::new(level).map_err(|error| InvalidGrayscaleCode(error.0))
    }
}

impl ApproxBrightness for Grayscale {
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        let soft_max = Self::MAX.level().into();
        Brightness::new(self.level().into()).spread_level(soft_max)
    }
}

impl MutableApproxBrightness for Grayscale {
    fn set_approx_brightness(
        &mut self,
        brightness: Brightness,
    ) -> Result<(), BrightnessError> {
        let compressed = brightness.compress_level(Self::MAX.level().into())?;
        let level = u8::try_from(compressed.level())?;
        *self = Self::new(level).map_err(BrightnessError::approximation)?;
        Ok(())
    }
}
