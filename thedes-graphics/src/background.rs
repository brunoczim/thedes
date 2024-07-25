use thedes_domain::matter::Ground;
use thedes_tui::color::{Color, EightBitColor, LegacyRgb};

pub trait EntityTile {
    fn base_color(&self) -> Color;
}

impl EntityTile for Ground {
    fn base_color(&self) -> Color {
        let cmy_color = match self {
            Ground::Sand => LegacyRgb::new(4, 4, 1),
            Ground::Grass => LegacyRgb::new(1, 5, 2),
            Ground::Stone => LegacyRgb::new(2, 2, 2),
        };

        EightBitColor::from(cmy_color).into()
    }
}
