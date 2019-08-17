use std::ops::{Add, Index, IndexMut, Sub};

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
pub type Coord = u16;

/// The excess on which position coordinates are encoded.
pub const ORIGIN_EXCESS: Coord = !0 - (!0 >> 1);

/// A coordinate that can index Vec2D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    /// The X (horizontal) axis.
    X,
    /// The Y (vertical) axis.
    Y,
}

impl Axis {
    /// A new empty iterator.
    pub fn iter() -> AxisIter {
        AxisIter::default()
    }

    pub fn orthogonal(self) -> Self {
        match self {
            Self::X => Self::Y,
            Self::Y => Self::X,
        }
    }
}

/// An iterator on all used axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AxisIter {
    curr: Option<Axis>,
}

impl Default for AxisIter {
    fn default() -> Self {
        Self { curr: Some(Axis::X) }
    }
}

impl Iterator for AxisIter {
    type Item = Axis;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr?;

        self.curr = match curr {
            Axis::X => Some(Axis::Y),
            Axis::Y => None,
        };

        Some(curr)
    }
}

/// A positioned rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rect {
    /// Top left coordinates (x, y) of this rectangle.
    pub start: Coord2D,
    /// The size of this rectangle.
    pub size: Coord2D,
}

impl Rect {
    /// Tests if a point is inside the rectangle.
    pub fn has_point(self, point: Coord2D) -> bool {
        Axis::iter().all(|axis| {
            point[axis] >= self.start[axis]
                && point[axis] < self.start[axis] + self.size[axis]
        })
    }

    fn has_corner_inside(self, other: Rect) -> bool {
        self.has_point(other.start)
            || self.has_point(other.start + other.size)
            || self.has_point(other.start + Coord2D { x: other.size.x, y: 0 })
            || self.has_point(other.start + Coord2D { x: 0, y: other.size.y })
    }

    /// Tests if the rectangles are overlapping.
    pub fn overlaps(self, other: Rect) -> bool {
        self.has_corner_inside(other) || other.has_corner_inside(self)
    }

    /// Tests if self moving from the origin crashes on other.
    pub fn moves_through(self, other: Rect, origin: Coord, axis: Axis) -> bool {
        let mut extended = self;

        extended.start[axis] = origin.min(self.start[axis]);
        extended.size[axis] =
            self.start[axis] - extended.start[axis] + self.size[axis];

        other.overlaps(extended)
    }
}

/// An array representing objects in a (bidimensional) plane, such as points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vec2D<T> {
    /// The object on X.
    pub x: T,
    /// The object on Y.
    pub y: T,
}

impl<T> Vec2D<T> {
    /// Creates a new Vec2D from a function mapping each axis to a value.
    pub fn from_map<F>(mut fun: F) -> Self
    where
        F: FnMut(Axis) -> T,
    {
        Self { x: fun(Axis::X), y: fun(Axis::Y) }
    }

    pub fn inv(self) -> Self {
        Self { x: self.y, y: self.x }
    }
}

impl<T> Index<Axis> for Vec2D<T> {
    type Output = T;

    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
        }
    }
}

impl<T> IndexMut<Axis> for Vec2D<T> {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        match axis {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
        }
    }
}

impl<T> Add<Self> for Vec2D<T>
where
    T: Add<T>,
{
    type Output = Vec2D<T::Output>;

    fn add(self, other: Self) -> Self::Output {
        Vec2D { x: self.x + other.x, y: self.y + other.y }
    }
}

impl<T> Sub<Self> for Vec2D<T>
where
    T: Sub<T>,
{
    type Output = Vec2D<T::Output>;

    fn sub(self, other: Self) -> Self::Output {
        Vec2D { x: self.x - other.x, y: self.y - other.y }
    }
}

/// A 2D coordinate object such as a point or the sides of an area.
pub type Coord2D = Vec2D<Coord>;

impl Coord2D {
    /// The origin on the plane, encoding the (0,0) position with excess.
    pub const ORIGIN: Self = Self { x: ORIGIN_EXCESS, y: ORIGIN_EXCESS };
}

/// NatPosinates of where the game Camera is showing.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Camera {
    pub rect: Rect,
}
