use crate::{
    graphics::BasicColor,
    math::plane::{Camera, Coord2, Nat},
    terminal,
};
use std::fmt;

/// A ground block (background color).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Ground {
    /// This block's ground is grass.
    Grass,
    /// This block's ground is sand.
    Sand,
    /// This block's ground is rock.
    Rock,
    /// This block's ground is a path.
    Path,
}

impl fmt::Display for Ground {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(match self {
            Ground::Grass => "grass",
            Ground::Sand => "sand",
            Ground::Rock => "rock",
            Ground::Path => "path",
        })
    }
}

impl Ground {
    /// Renders this ground type on the screen.
    pub fn render(
        &self,
        pos: Coord2<Nat>,
        camera: Camera,
        screen: &mut terminal::Screen,
    ) {
        if let Some(pos) = camera.convert(pos) {
            let fg = screen.get(pos).clone().fg();
            let bg = match self {
                Ground::Grass => BasicColor::LightGreen.into(),
                Ground::Sand => BasicColor::LightYellow.into(),
                Ground::Rock => BasicColor::LightGrey.into(),
                Ground::Path => BasicColor::DarkYellow.into(),
            };
            screen.set(pos, fg.make_tile(bg));
        }
    }
}
