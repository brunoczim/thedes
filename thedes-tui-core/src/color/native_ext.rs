use super::{BasicColor, Color, LegacyRgb, grayscale::Grayscale, rgb::Rgb};

pub(crate) trait ColorToCrossterm {
    fn to_crossterm(&self) -> crossterm::style::Color;
}

impl ColorToCrossterm for Color {
    fn to_crossterm(&self) -> crossterm::style::Color {
        match self {
            Self::Basic(color) => color.to_crossterm(),
            Self::LegacyRgb(color) => color.to_crossterm(),
            Self::Rgb(color) => color.to_crossterm(),
            Self::Grayscale(color) => color.to_crossterm(),
        }
    }
}

impl ColorToCrossterm for BasicColor {
    fn to_crossterm(&self) -> crossterm::style::Color {
        match self {
            Self::Black => crossterm::style::Color::Black,
            Self::DarkRed => crossterm::style::Color::DarkRed,
            Self::DarkGreen => crossterm::style::Color::DarkGreen,
            Self::DarkYellow => crossterm::style::Color::DarkYellow,
            Self::DarkBlue => crossterm::style::Color::DarkBlue,
            Self::DarkMagenta => crossterm::style::Color::DarkMagenta,
            Self::DarkCyan => crossterm::style::Color::DarkCyan,
            Self::LightGray => crossterm::style::Color::Grey,
            Self::DarkGray => crossterm::style::Color::DarkGrey,
            Self::LightRed => crossterm::style::Color::Red,
            Self::LightGreen => crossterm::style::Color::Green,
            Self::LightYellow => crossterm::style::Color::Yellow,
            Self::LightBlue => crossterm::style::Color::Blue,
            Self::LightMagenta => crossterm::style::Color::Magenta,
            Self::LightCyan => crossterm::style::Color::Cyan,
            Self::White => crossterm::style::Color::White,
        }
    }
}

impl ColorToCrossterm for LegacyRgb {
    fn to_crossterm(&self) -> crossterm::style::Color {
        crossterm::style::Color::AnsiValue(self.code())
    }
}

impl ColorToCrossterm for Rgb {
    fn to_crossterm(&self) -> crossterm::style::Color {
        crossterm::style::Color::Rgb {
            r: self.red,
            g: self.green,
            b: self.blue,
        }
    }
}

impl ColorToCrossterm for Grayscale {
    fn to_crossterm(&self) -> crossterm::style::Color {
        crossterm::style::Color::AnsiValue(self.code())
    }
}
