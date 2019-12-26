use std::ops::{Add, Index, IndexMut, Sub};

/// A direction on the screen.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
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

/// Type alias to a signed integer position, a coordinate.
pub type ICoord = i16;

/// The excess on which position coordinates are encoded.
pub const ORIGIN_EXCESS: Coord = !0 - (!0 >> 1);

/// A coordinate that can index Vec2D.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
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

    pub fn next_axis(self) -> Self {
        match self {
            Axis::X => Axis::Y,
            Axis::Y => Axis::X,
        }
    }
}

/// An iterator on all used axis.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
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
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Rect {
    /// Top left coordinates (x, y) of this rectangle.
    pub start: Coord2D,
    /// The size of this rectangle.
    pub size: Coord2D,
}

impl Rect {
    /// Calculates and returns the end point (bottom-right) of this rectangle.
    pub fn end(self) -> Coord2D {
        Coord2D::from_map(|axis| self.start[axis] + self.size[axis])
    }

    /// Tests if a point is inside the rectangle.
    pub fn has_point(self, point: Coord2D) -> bool {
        Axis::iter().all(|axis| {
            point[axis] >= self.start[axis] && point[axis] < self.end()[axis]
        })
    }

    /// Tests if the rectangles are overlapping.
    pub fn overlaps(self, other: Rect) -> bool {
        Axis::iter().all(|axis| {
            self.start[axis] <= other.end()[axis]
                && other.start[axis] <= self.end()[axis]
        })
    }

    /// Returns the overlapped area of the given rectangles.
    pub fn overlapped(self, other: Rect) -> Option<Rect> {
        let start =
            Coord2D::from_map(|axis| self.start[axis].max(other.start[axis]));

        let maybe_size = Vec2D::from_map(|axis| {
            self.end()[axis].min(other.end()[axis]).checked_sub(start[axis])
        });

        let size = Coord2D { x: maybe_size.x?, y: maybe_size.y? };

        Some(Rect { start, size })
    }

    /// Tests if self moving from the origin crashes on other.
    pub fn moves_through(self, other: Rect, origin: Coord, axis: Axis) -> bool {
        let mut extended = self;

        extended.start[axis] = origin.min(self.start[axis]);
        extended.size[axis] =
            self.start[axis] - extended.start[axis] + self.size[axis];

        other.overlaps(extended)
    }

    /// Tests if the size of this rectangle overflows when added to the
    /// coordinates.
    pub fn size_overflows(self) -> bool {
        self.start.x.checked_add(self.size.x).is_none()
            || self.start.y.checked_add(self.size.y).is_none()
    }
}

/// An array representing objects in a (bidimensional) plane, such as points.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
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

    /// Inverts the vector coordinates: x becomes y, y becomes x.
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

    /// Moves this coordinate by one unity in the given direction.
    pub fn move_by_direc(self, direc: Direc) -> Self {
        match direc {
            Direc::Up => Self { y: self.y.saturating_sub(1), ..self },
            Direc::Down => Self { y: self.y.saturating_add(1), ..self },
            Direc::Left => Self { x: self.x.saturating_sub(1), ..self },
            Direc::Right => Self { x: self.x.saturating_add(1), ..self },
        }
    }

    /// Converts unsigned coordinates to signed coordinates, relative to the
    /// origin, so it can be presented to the player.
    pub fn printable_pos(self) -> Vec2D<ICoord> {
        Vec2D {
            x: self.x.wrapping_sub(ORIGIN_EXCESS) as ICoord,
            y: ORIGIN_EXCESS.wrapping_sub(self.y) as ICoord,
        }
    }
}

/*
/// NatPosinates of where the game Camera is showing.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Camera {
    pub rect: Rect,
}

impl Camera {
    pub fn make_context<'output, B>(
        self,
        node: Rect,
        error: &'output mut GameResult<()>,
        term: &'output mut Terminal<B>,
    ) -> Option<Context<'output, B>>
    where
        B: Backend,
    {
        self.rect.overlapped(node).map(move |overlapped| {
            let crop = Rect {
                start: Coord2D::from_map(|axis| {
                    overlapped.start[axis] - node.start[axis]
                }),

                size: overlapped.size,
            };

            let screen = Coord2D::from_map(|axis| {
                overlapped.start[axis] - self.rect.start[axis]
            });

            Context::new(error, term, crop, screen)
        })
    }
}*/

pub trait Positioned {
    fn top_left(&self) -> Coord2D;
}

#[cfg(test)]
mod test {
    use super::{Coord2D, Rect};

    #[test]
    fn overlapped_area() {
        let rect1 = Rect {
            start: Coord2D { x: 0, y: 0 },
            size: Coord2D { x: 5, y: 5 },
        };

        let rect2 = Rect {
            start: Coord2D { x: 6, y: 0 },
            size: Coord2D { x: 5, y: 5 },
        };

        let rect3 = Rect {
            start: Coord2D { x: 5, y: 0 },
            size: Coord2D { x: 5, y: 5 },
        };

        let rect4 = Rect {
            start: Coord2D { x: 1, y: 1 },
            size: Coord2D { x: 3, y: 3 },
        };

        assert_eq!(rect1.overlapped(rect2), None);
        assert_eq!(
            rect1.overlapped(rect3),
            Some(Rect {
                start: Coord2D { x: 5, y: 0 },
                size: Coord2D { x: 0, y: 5 }
            })
        );
        assert_eq!(rect1.overlapped(rect4), Some(rect4));
    }
}
