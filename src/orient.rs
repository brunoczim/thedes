use std::ops::{Index, IndexMut};

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

/// Type alias to a natural number (unsigned integer) position, a coordinate.
pub type NatPos = u16;
/// Type alias to a (signed) integer position, a coordinate.
pub type IntPos = i16;

/// A coordinate that can index Vec2d.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    /// The X (horizontal) axis.
    X,
    /// The Y (vertical) axis.
    Y,
}

/// A positioned rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rect<P> {
    /// Top left coordinates (x, y).
    pub start: Point<P>,
    /// The size of this line.
    pub size: Vec2d<NatPos>,
}

/// An array representing objects in a (bidimensional) plane, such as points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vec2d<P> {
    /// The object on X.
    pub x: P,
    /// The object on Y.
    pub y: P,
}

impl<P> Index<Axis> for Vec2d<P> {
    type Output = P;

    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
        }
    }
}

impl<P> IndexMut<Axis> for Vec2d<P> {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        match axis {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
        }
    }
}

/// Vec2d as a point, used for clarity.
pub type Point<P> = Vec2d<P>;

/// NatPosinates of where the game Camera is showing.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Camera {
    pub rect: Rect<IntPos>,
}

impl Camera {
    /// Resolves a pair of coordinates into screen coordinates, if they are
    /// inside of the camera.
    pub fn resolve(self, x: IntPos, y: IntPos) -> Option<(NatPos, NatPos)> {
        let dx = x - self.rect.start.x;
        let dy = y - self.rect.start.y;

        if dx >= 0
            && self.rect.size.x > dx as NatPos
            && dy >= 0
            && self.rect.size.y > dy as NatPos
        {
            Some((dx as NatPos, dy as NatPos))
        } else {
            None
        }
    }
}
