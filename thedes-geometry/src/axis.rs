use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    Y,
    X,
}

impl Axis {
    pub const ALL: [Self; 2] = [Self::Y, Self::X];
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Y => write!(f, "y"),
            Self::X => write!(f, "x"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::Up, Self::Left, Self::Down, Self::Right];
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Up => write!(f, "-y"),
            Self::Left => write!(f, "-x"),
            Self::Down => write!(f, "+y"),
            Self::Right => write!(f, "+x"),
        }
    }
}
