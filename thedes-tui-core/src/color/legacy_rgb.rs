use thiserror::Error;

use super::{
    ApproxBrightness,
    Brightness,
    BrightnessError,
    MutableApproxBrightness,
    channel_vector::{
        self,
        BLUE_MILLI_WEIGHT,
        Channel,
        ChannelValue,
        ChannelVector,
        GREEN_MILLI_WEIGHT,
        RED_MILLI_WEIGHT,
    },
};

#[derive(Debug, Clone, Copy, Error)]
#[error("Color code {0} is invalid for legacy RGB")]
pub struct BadLegacyRgbCode(pub u8);

#[derive(Debug, Clone, Copy, Error)]
#[error("Color code {0} is invalid for legacy RGB")]
pub struct BadLegacyLevel(pub u8);

const CODE_OFFSET: u8 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum LegacyLevel {
    #[default]
    L0 = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
    L5 = 5,
}

impl LegacyLevel {
    pub const MAX: Self = Self::L5;

    pub const SIZE: u8 = 6;

    pub fn code(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for LegacyLevel {
    type Error = BadLegacyLevel;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        Ok(match code {
            0 => Self::L0,
            1 => Self::L1,
            2 => Self::L2,
            3 => Self::L3,
            4 => Self::L4,
            5 => Self::L5,
            _ => Err(BadLegacyLevel(code))?,
        })
    }
}

impl From<LegacyLevel> for u8 {
    fn from(level: LegacyLevel) -> Self {
        level.code()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LegacyRgb {
    pub red: LegacyLevel,
    pub green: LegacyLevel,
    pub blue: LegacyLevel,
}

impl Default for LegacyRgb {
    fn default() -> Self {
        Self::BLACK
    }
}

impl LegacyRgb {
    pub const BLACK: Self = Self {
        red: LegacyLevel::L0,
        green: LegacyLevel::L0,
        blue: LegacyLevel::L0,
    };

    pub fn new(
        red_code: u8,
        green_code: u8,
        blue_code: u8,
    ) -> Result<Self, BadLegacyLevel> {
        Ok(Self {
            red: LegacyLevel::try_from(red_code)?,
            green: LegacyLevel::try_from(green_code)?,
            blue: LegacyLevel::try_from(blue_code)?,
        })
    }

    pub fn code(self) -> u8 {
        CODE_OFFSET
            + self.red.code() * (LegacyLevel::SIZE * LegacyLevel::SIZE)
            + self.green.code() * LegacyLevel::SIZE
            + self.blue.code()
    }

    fn to_channel_buf(self) -> Result<[Channel; 3], channel_vector::Error> {
        Ok([
            Channel::new(
                ChannelValue::from(self.red.code()),
                RED_MILLI_WEIGHT,
            )?,
            Channel::new(
                ChannelValue::from(self.green.code()),
                GREEN_MILLI_WEIGHT,
            )?,
            Channel::new(
                ChannelValue::from(self.blue.code()),
                BLUE_MILLI_WEIGHT,
            )?,
        ])
    }
}

impl TryFrom<u8> for LegacyRgb {
    type Error = BadLegacyRgbCode;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        let Some(relative) = code.checked_sub(CODE_OFFSET) else {
            Err(BadLegacyRgbCode(code))?
        };

        let red_code = relative / (LegacyLevel::SIZE * 2) % LegacyLevel::SIZE;
        let green_code = relative / LegacyLevel::SIZE % LegacyLevel::SIZE;
        let blue_code = relative % LegacyLevel::SIZE;

        Self::new(red_code, green_code, blue_code)
            .map_err(|_| BadLegacyRgbCode(code))
    }
}

impl ApproxBrightness for LegacyRgb {
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        let mut buf =
            self.to_channel_buf().map_err(BrightnessError::approximation)?;
        ChannelVector::new(&mut buf, ChannelValue::from(LegacyLevel::SIZE - 1))
            .map_err(BrightnessError::approximation)?
            .approx_brightness()
    }
}

impl MutableApproxBrightness for LegacyRgb {
    fn set_approx_brightness(
        &mut self,
        brightness: Brightness,
    ) -> Result<(), BrightnessError> {
        let mut buf =
            self.to_channel_buf().map_err(BrightnessError::approximation)?;
        ChannelVector::new(&mut buf, ChannelValue::from(LegacyLevel::SIZE - 1))
            .map_err(BrightnessError::approximation)?
            .set_approx_brightness(brightness)?;
        let result = Self::new(
            u8::try_from(buf[0].value())?,
            u8::try_from(buf[1].value())?,
            u8::try_from(buf[2].value())?,
        );
        *self = result.map_err(BrightnessError::approximation)?;
        Ok(())
    }
}
