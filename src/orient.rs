/// A direction on the screen.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direc {
    /// Going up (-y).
    Up,
    /// Going left (-x).
    Left,
    /// Going down (+y).
    Down,
    /// Going right (+x).
    Right,
}

/// Type alias to an unsigned integer representing a coordinate.
pub type Coord = u16;
/// Type alias to a signed integer with same size qs Coord.
pub type ICoord = i16;

/// Coordinates of where the game Camera is showing.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Camera {
    /// X coordinate of camera's top-left.
    pub x: ICoord,
    /// Y coordinate of camera's top-left.
    pub y: ICoord,
    /// Width of the camera.
    pub width: Coord,
    /// Height of the camera.
    pub height: Coord,
}

impl Camera {
    /// Resolves a pair of coordinates into screen coordinates, if they are
    /// inside of the camera.
    pub fn resolve(self, x: ICoord, y: ICoord) -> Option<(Coord, Coord)> {
        let dx = x - self.x;
        let dy = y - self.y;

        if dx >= 0
            && self.width > dx as Coord
            && dy >= 0
            && self.height > dy as Coord
        {
            Some((dx as Coord, dy as Coord))
        } else {
            None
        }
    }
}
