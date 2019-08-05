/// A direction on the screen.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direc {
    /// Going up (-y)
    Up,
    /// Going left (-x)
    Left,
    /// Going down (+y)
    Down,
    /// Going right (+x)
    Right
}

/// Type alias to an unsigned integer representing a coordinate.
pub type Coord = u16;
/// Type alias to a signed integer with same size qs Coord.
pub type ICoord = i16;

