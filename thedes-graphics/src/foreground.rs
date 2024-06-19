use thedes_geometry::axis::Direction;
use thedes_tui::{
    color::{Color, EightBitColor, LegacyRgb},
    grapheme::{self, NotGrapheme},
};

pub trait EntityTile {
    fn base_color(&self) -> Color;

    fn grapheme(
        &self,
        graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PlayerHead;

impl EntityTile for PlayerHead {
    fn base_color(&self) -> Color {
        EightBitColor::from(LegacyRgb::new(0, 0, 0)).into()
    }

    fn grapheme(
        &self,
        graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme> {
        graphemes.get_or_register("o")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerPointer {
    pub facing: Direction,
}

impl EntityTile for PlayerPointer {
    fn base_color(&self) -> Color {
        EightBitColor::from(LegacyRgb::new(0, 0, 0)).into()
    }

    fn grapheme(
        &self,
        graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme> {
        let grapheme = match self.facing {
            Direction::Up => "Λ",
            Direction::Left => "<",
            Direction::Down => "V",
            Direction::Right => ">",
        };
        graphemes.get_or_register(grapheme)
    }
}
