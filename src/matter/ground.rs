use crate::{
    graphics::CMYColor,
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
    DebugArea,
    DebugVertex,
    DebugDoor,
}

impl fmt::Display for Ground {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(match self {
            Ground::Grass => "grass",
            Ground::Sand => "sand",
            Ground::Rock => "rock",
            Ground::Path => "path",
            Ground::DebugArea => "dbgare",
            Ground::DebugVertex => "dbgvtx",
            Ground::DebugDoor => "dbgdor",
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
                Ground::Grass => CMYColor::new(2, 4, 0).into(),
                Ground::Sand => CMYColor::new(5, 4, 1).into(),
                Ground::Rock => CMYColor::new(3, 2, 1).into(),
                Ground::Path => CMYColor::new(3, 3, 3).into(),
                Ground::DebugArea => CMYColor::new(0, 0, 0).into(),
                Ground::DebugVertex => CMYColor::new(5, 5, 5).into(),
                Ground::DebugDoor => CMYColor::new(2, 2, 5).into(),
            };
            screen.set(pos, fg.make_tile(bg));
        }
    }
}
