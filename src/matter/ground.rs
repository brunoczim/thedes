use crate::{map::Coord, session::Camera};
use andiskaz::{color::CmyColor, screen::Screen, tile::Background};
use gardiz::coord::Vec2;
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
        pos: Vec2<Coord>,
        camera: Camera,
        screen: &mut Screen,
    ) {
        if let Some(pos) = camera.convert(pos) {
            let color = match self {
                Ground::Grass => CmyColor::new(2, 4, 0),
                Ground::Sand => CmyColor::new(5, 5, 1),
                Ground::Rock => CmyColor::new(3, 2, 1),
                Ground::Path => CmyColor::new(3, 3, 3),
            };
            screen.set(pos, Background { color: color.into() });
        }
    }
}
