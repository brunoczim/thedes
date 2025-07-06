use thiserror::Error;

use super::{
    ApproxBrightness,
    Brightness,
    BrightnessError,
    MutableApproxBrightness,
    channel_vector::{BLUE_MILLI_WEIGHT, GREEN_MILLI_WEIGHT, RED_MILLI_WEIGHT},
};

#[derive(Debug, Error)]
#[error("Code {0} is not valid for basic colors")]
pub struct BadBassicColorCode(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum BasicColorCore {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum BasicColorVariant {
    Dark = 0,
    Light = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BasicColorParts {
    pub core: BasicColorCore,
    pub variant: BasicColorVariant,
}

impl BasicColorParts {
    pub fn encode(self) -> BasicColor {
        match self.variant {
            BasicColorVariant::Dark => match self.core {
                BasicColorCore::Black => BasicColor::Black,
                BasicColorCore::Red => BasicColor::DarkRed,
                BasicColorCore::Green => BasicColor::DarkGreen,
                BasicColorCore::Yellow => BasicColor::DarkYellow,
                BasicColorCore::Blue => BasicColor::DarkBlue,
                BasicColorCore::Magenta => BasicColor::DarkMagenta,
                BasicColorCore::Cyan => BasicColor::DarkCyan,
                BasicColorCore::White => BasicColor::LightGray,
            },
            BasicColorVariant::Light => match self.core {
                BasicColorCore::Black => BasicColor::DarkGray,
                BasicColorCore::Red => BasicColor::LightRed,
                BasicColorCore::Green => BasicColor::LightGreen,
                BasicColorCore::Yellow => BasicColor::LightYellow,
                BasicColorCore::Blue => BasicColor::LightBlue,
                BasicColorCore::Magenta => BasicColor::LightMagenta,
                BasicColorCore::Cyan => BasicColor::LightCyan,
                BasicColorCore::White => BasicColor::White,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum BasicColor {
    Black = 0,
    DarkRed = 1,
    DarkGreen = 2,
    DarkYellow = 3,
    DarkBlue = 4,
    DarkMagenta = 5,
    DarkCyan = 6,
    LightGray = 7,
    DarkGray = 8,
    LightRed = 9,
    LightGreen = 10,
    LightYellow = 11,
    LightBlue = 12,
    LightMagenta = 13,
    LightCyan = 14,
    White = 15,
}

impl Default for BasicColor {
    fn default() -> Self {
        Self::Black
    }
}

impl BasicColor {
    const BLACK_CODE: u8 = Self::Black as _;
    const DARK_RED_CODE: u8 = Self::DarkRed as _;
    const DARK_GREEN_CODE: u8 = Self::DarkGreen as _;
    const DARK_YELLOW_CODE: u8 = Self::DarkYellow as _;
    const DARK_BLUE_CODE: u8 = Self::DarkBlue as _;
    const DARK_MAGENTA_CODE: u8 = Self::DarkMagenta as _;
    const DARK_CYAN_CODE: u8 = Self::DarkCyan as _;
    const LIGHT_GRAY_CODE: u8 = Self::LightGray as _;
    const DARK_GRAY_CODE: u8 = Self::DarkGray as _;
    const LIGHT_RED_CODE: u8 = Self::LightRed as _;
    const LIGHT_GREEN_CODE: u8 = Self::LightGreen as _;
    const LIGHT_YELLOW_CODE: u8 = Self::LightYellow as _;
    const LIGHT_BLUE_CODE: u8 = Self::LightBlue as _;
    const LIGHT_MAGENTA_CODE: u8 = Self::LightMagenta as _;
    const LIGHT_CYAN_CODE: u8 = Self::LightCyan as _;
    const WHITE_CODE: u8 = Self::White as _;

    pub fn new(color_core: BasicColorCore, variant: BasicColorVariant) -> Self {
        Self::encode_parts(BasicColorParts { core: color_core, variant })
    }

    pub fn encode_parts(parts: BasicColorParts) -> Self {
        parts.encode()
    }

    pub fn decode_parts(self) -> BasicColorParts {
        let (variant, core) = match self {
            Self::Black => (BasicColorVariant::Dark, BasicColorCore::Black),
            Self::DarkRed => (BasicColorVariant::Dark, BasicColorCore::Red),
            Self::DarkGreen => (BasicColorVariant::Dark, BasicColorCore::Green),
            Self::DarkYellow => {
                (BasicColorVariant::Dark, BasicColorCore::Yellow)
            },
            Self::DarkBlue => (BasicColorVariant::Dark, BasicColorCore::Blue),
            Self::DarkMagenta => {
                (BasicColorVariant::Dark, BasicColorCore::Magenta)
            },
            Self::DarkCyan => (BasicColorVariant::Dark, BasicColorCore::Cyan),
            Self::LightGray => (BasicColorVariant::Dark, BasicColorCore::White),
            Self::DarkGray => (BasicColorVariant::Light, BasicColorCore::Black),
            Self::LightRed => (BasicColorVariant::Light, BasicColorCore::Red),
            Self::LightGreen => {
                (BasicColorVariant::Light, BasicColorCore::Green)
            },
            Self::LightYellow => {
                (BasicColorVariant::Light, BasicColorCore::Yellow)
            },
            Self::LightBlue => (BasicColorVariant::Light, BasicColorCore::Blue),
            Self::LightMagenta => {
                (BasicColorVariant::Light, BasicColorCore::Magenta)
            },
            Self::LightCyan => (BasicColorVariant::Light, BasicColorCore::Cyan),
            Self::White => (BasicColorVariant::Light, BasicColorCore::White),
        };

        BasicColorParts { variant, core }
    }
}

impl TryFrom<u8> for BasicColor {
    type Error = BadBassicColorCode;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        let decoded = match code {
            Self::BLACK_CODE => Self::Black,
            Self::DARK_RED_CODE => Self::DarkRed,
            Self::DARK_GREEN_CODE => Self::DarkGreen,
            Self::DARK_YELLOW_CODE => Self::DarkYellow,
            Self::DARK_BLUE_CODE => Self::DarkBlue,
            Self::DARK_MAGENTA_CODE => Self::DarkMagenta,
            Self::DARK_CYAN_CODE => Self::DarkCyan,
            Self::LIGHT_GRAY_CODE => Self::LightGray,
            Self::DARK_GRAY_CODE => Self::DarkGray,
            Self::LIGHT_RED_CODE => Self::LightRed,
            Self::LIGHT_GREEN_CODE => Self::LightGreen,
            Self::LIGHT_YELLOW_CODE => Self::LightYellow,
            Self::LIGHT_BLUE_CODE => Self::LightBlue,
            Self::LIGHT_MAGENTA_CODE => Self::LightMagenta,
            Self::LIGHT_CYAN_CODE => Self::LightCyan,
            Self::WHITE_CODE => Self::White,
            _ => Err(BadBassicColorCode(code))?,
        };
        Ok(decoded)
    }
}

impl ApproxBrightness for BasicColor {
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        let red_weight = RED_MILLI_WEIGHT;
        let green_weight = GREEN_MILLI_WEIGHT;
        let blue_weight = BLUE_MILLI_WEIGHT;
        let total_base = red_weight + green_weight + blue_weight;
        let soft_max = total_base * 2 - 1;

        let parts = self.decode_parts();
        let base = match parts.core {
            BasicColorCore::Black => 0,
            BasicColorCore::Red => red_weight * 2,
            BasicColorCore::Green => green_weight * 2,
            BasicColorCore::Yellow => (red_weight + green_weight) / 2,
            BasicColorCore::Blue => blue_weight * 2,
            BasicColorCore::Magenta => (red_weight + blue_weight) / 2,
            BasicColorCore::Cyan => (blue_weight + green_weight) / 2,
            BasicColorCore::White => soft_max * 3 / 4,
        };

        let light_bonus = match parts.variant {
            BasicColorVariant::Dark => 0,
            BasicColorVariant::Light => total_base,
        };

        let in_basic_color_space = (base + light_bonus).min(soft_max);

        Brightness::new(in_basic_color_space).spread_level(soft_max)
    }
}

impl MutableApproxBrightness for BasicColor {
    fn set_approx_brightness(
        &mut self,
        brightness: Brightness,
    ) -> Result<(), BrightnessError> {
        let parts = self.decode_parts();

        let light_variant = Self::encode_parts(BasicColorParts {
            variant: BasicColorVariant::Light,
            ..parts
        });

        let dark_variant = Self::encode_parts(BasicColorParts {
            variant: BasicColorVariant::Dark,
            ..parts
        });

        let light_brightness = light_variant.approx_brightness()?;

        *self = if brightness >= light_brightness {
            light_variant
        } else {
            dark_variant
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::color::{ApproxBrightness, MutableApproxBrightness};

    use super::BasicColor;

    #[test]
    fn integer_encode_decode_complement() {
        for code in 0_u8 .. 16 {
            let color = BasicColor::try_from(code).unwrap();
            let encoded = color as u8;

            assert_eq!(
                encoded, code,
                "Mismatch between original code and re-encoded, decoded color \
                 is: {:?}",
                color,
            );
        }
    }

    #[test]
    fn decode_out_of_bounds() {
        BasicColor::try_from(16_u8).unwrap_err();
        BasicColor::try_from(17_u8).unwrap_err();
        BasicColor::try_from(20_u8).unwrap_err();
    }

    #[test]
    fn parts_encode_decode_complement() {
        for code in 0_u8 .. 16 {
            let color = BasicColor::try_from(code).unwrap();
            let parts = color.decode_parts();
            let encoded_color = parts.encode();

            assert_eq!(
                encoded_color, color,
                "Mismatch between original color and re-encoded, decoded \
                 parts are: {:?}",
                parts,
            );
        }
    }

    #[test]
    fn assigning_its_own_brightness_preserves_itself() {
        for code in 0_u8 .. 16 {
            let color = BasicColor::try_from(code).unwrap();
            let brightness = color.approx_brightness().unwrap();
            let transformed = color.with_approx_brightness(brightness).unwrap();
            assert_eq!(color, transformed);
        }
    }
}
