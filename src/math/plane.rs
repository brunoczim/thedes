use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{
    cmp::{Ord, Ordering},
    ops::{Add, Div, Index, IndexMut, Mul, Neg, Not, Rem, Sub},
};

/// Defines fixed width unsigned integer used for natural numbers.
pub type Nat = u16;

/// Defines fixed width signed integer used for plain integers.
pub type Int = i16;

/// The excess on which position coordinates are encoded.
pub const ORIGIN_EXCESS: Nat = Nat::max_value() - (!0 >> 1);

/// Labels over axes used by the [Coord2] type.
#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(u8)]
pub enum Axis {
    /// "Horizontal" axis label.
    X,
    /// "Vertical" axis label.
    Y,
}

impl Axis {
    /// The number of dimensions.
    pub const COUNT: usize = 2;

    /// Creates iterator that yields all the axis labels (X, Y).
    pub fn iter() -> AxisIter {
        AxisIter { curr: Some(Axis::X) }
    }
}

impl Not for Axis {
    type Output = Axis;

    fn not(self) -> Self::Output {
        match self {
            Axis::X => Axis::Y,
            Axis::Y => Axis::X,
        }
    }
}

impl Distribution<Axis> for Standard {
    fn sample<R>(&self, rng: &mut R) -> Axis
    where
        R: Rng + ?Sized,
    {
        let arr = [Axis::X, Axis::Y];
        let index = if rng.gen::<bool>() { 1 } else { 0 };
        arr[index]
    }
}

/// Iterator over axes labels.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AxisIter {
    curr: Option<Axis>,
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

/// A point of generic elements. `Coord2<Nat>` is used by the terminal.
#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Coord2<T> {
    /// The "vertical" axis value.
    pub y: T,
    /// The "horizontal" axis value.
    pub x: T,
}

impl<T> Index<Axis> for Coord2<T> {
    type Output = T;

    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
        }
    }
}

impl<T> IndexMut<Axis> for Coord2<T> {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        match axis {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
        }
    }
}

impl<T> Not for Coord2<T> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Coord2 { x: self.y, y: self.x }
    }
}

impl<T> Coord2<T> {
    /// Creates a [Coord2] by mapping an axis to a value.
    pub fn from_axes<F>(mut fun: F) -> Self
    where
        F: FnMut(Axis) -> T,
    {
        Self { x: fun(Axis::X), y: fun(Axis::Y) }
    }

    /// Converts the element type into references, so that the point can use
    /// owneership-taking methods without actually moving the point.
    pub fn as_ref(&self) -> Coord2<&T> {
        Coord2 { x: &self.x, y: &self.y }
    }

    /// Converts the element type into mutable references, so that the point can
    /// use owneership-taking methods without actually moving the point and be
    /// mutated.
    pub fn as_mut(&mut self) -> Coord2<&mut T> {
        Coord2 { x: &mut self.x, y: &mut self.y }
    }

    /// Maps each component of the point into a new point of another type by
    /// using a function.
    pub fn map<F, U>(self, mut fun: F) -> Coord2<U>
    where
        F: FnMut(T) -> U,
    {
        Coord2 { x: fun(self.x), y: fun(self.y) }
    }

    /// Maps each component of the point into a new point of another type by
    /// using a function, which in this method also receives which axis it is.
    pub fn map_with_axis<F, U>(self, mut fun: F) -> Coord2<U>
    where
        F: FnMut(Axis, T) -> U,
    {
        Coord2 { x: fun(Axis::X, self.x), y: fun(Axis::Y, self.y) }
    }

    /// Starts with an initial "state" value, and using a function, it
    /// accumulates this state with a point component until all components are
    /// used, and then it returns the state. It starts on X.
    pub fn foldl<F, U>(self, init: U, mut fun: F) -> U
    where
        F: FnMut(U, T) -> U,
    {
        let Self { x, y } = self;
        let val = fun(init, x);
        fun(val, y)
    }

    /// Starts with an initial "state" value, and using a function, it
    /// accumulates this state with a point component until all components are
    /// used, and then it returns the state. It starts on Y.
    pub fn foldr<F, U>(self, init: U, mut fun: F) -> U
    where
        F: FnMut(T, U) -> U,
    {
        let Self { x, y } = self;
        let val = fun(y, init);
        fun(x, val)
    }

    /// Zips the content of two points into a new point of tuples.
    pub fn zip<U>(self, other: Coord2<U>) -> Coord2<(T, U)> {
        self.zip_with(other, |x, y| (x, y))
    }

    /// Zips the content of two points into a new point of content determined by
    /// a function.
    pub fn zip_with<F, U, V>(self, other: Coord2<U>, mut fun: F) -> Coord2<V>
    where
        F: FnMut(T, U) -> V,
    {
        Coord2 { x: fun(self.x, other.x), y: fun(self.y, other.y) }
    }
}

impl<T> Coord2<Option<T>> {
    /// Returns `Some` if all coordinates are `Some`, otherwise `None`.
    pub fn transpose(self) -> Option<Coord2<T>> {
        match (self.x, self.y) {
            (Some(x), Some(y)) => Some(Coord2 { x, y }),
            _ => None,
        }
    }
}

impl<T, U> Add<Coord2<U>> for Coord2<T>
where
    T: Add<U>,
{
    type Output = Coord2<T::Output>;

    fn add(self, other: Coord2<U>) -> Self::Output {
        self.zip_with(other, |a, b| a + b)
    }
}

impl<T, U> Sub<Coord2<U>> for Coord2<T>
where
    T: Sub<U>,
{
    type Output = Coord2<T::Output>;

    fn sub(self, other: Coord2<U>) -> Self::Output {
        self.zip_with(other, |a, b| a - b)
    }
}

impl<T, U> Mul<Coord2<U>> for Coord2<T>
where
    T: Mul<U>,
{
    type Output = Coord2<T::Output>;

    fn mul(self, other: Coord2<U>) -> Self::Output {
        self.zip_with(other, |a, b| a * b)
    }
}

impl<T, U> Div<Coord2<U>> for Coord2<T>
where
    T: Div<U>,
{
    type Output = Coord2<T::Output>;

    fn div(self, other: Coord2<U>) -> Self::Output {
        self.zip_with(other, |a, b| a / b)
    }
}

impl<T, U> Rem<Coord2<U>> for Coord2<T>
where
    T: Rem<U>,
{
    type Output = Coord2<T::Output>;

    fn rem(self, other: Coord2<U>) -> Self::Output {
        self.zip_with(other, |a, b| a % b)
    }
}

impl<T> Neg for Coord2<T>
where
    T: Neg,
{
    type Output = Coord2<T::Output>;

    fn neg(self) -> Self::Output {
        self.map(|a| -a)
    }
}

impl Coord2<Nat> {
    /// The origin on the plane, encoding the (0,0) position with excess.
    pub const ORIGIN: Self = Self { x: ORIGIN_EXCESS, y: ORIGIN_EXCESS };

    /// Moves this coordinate by one unity in the given direction.
    pub fn move_by_direc(self, direc: Direc) -> Option<Self> {
        let this = match direc {
            Direc::Up => Self { y: self.y.checked_sub(1)?, ..self },
            Direc::Down => Self { y: self.y.checked_add(1)?, ..self },
            Direc::Left => Self { x: self.x.checked_sub(1)?, ..self },
            Direc::Right => Self { x: self.x.checked_add(1)?, ..self },
        };

        Some(this)
    }

    /// Converts unsigned coordinates to signed coordinates, relative to the
    /// origin, so it can be presented to the player.
    pub fn printable_pos(self) -> Coord2<Int> {
        Coord2 {
            x: self.x.wrapping_sub(ORIGIN_EXCESS) as Int,
            y: ORIGIN_EXCESS.wrapping_sub(self.y) as Int,
        }
    }

    /// Computes the straight direction to another point, if it exists.
    pub fn straight_direc_to(self, other: Self) -> Option<Direc> {
        match self.zip_with(other, |a, b| a.cmp(&b)) {
            Coord2 { x: Ordering::Equal, y: Ordering::Greater } => {
                Some(Direc::Up)
            },
            Coord2 { x: Ordering::Equal, y: Ordering::Less } => {
                Some(Direc::Down)
            },
            Coord2 { x: Ordering::Greater, y: Ordering::Equal } => {
                Some(Direc::Left)
            },
            Coord2 { x: Ordering::Less, y: Ordering::Equal } => {
                Some(Direc::Right)
            },
            _ => None,
        }
    }
}

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
#[repr(u8)]
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

impl Direc {
    /// The number of directions.
    pub const COUNT: usize = 4;

    /// Iterator over all directions.
    pub fn iter() -> DirecIter {
        DirecIter { curr: Some(Direc::Up) }
    }

    /// The axis on which this direction varies on.
    pub fn axis(self) -> Axis {
        match self {
            Direc::Up | Direc::Down => Axis::Y,
            Direc::Left | Direc::Right => Axis::X,
        }
    }
}

impl Not for Direc {
    type Output = Direc;

    fn not(self) -> Self::Output {
        match self {
            Direc::Up => Direc::Down,
            Direc::Down => Direc::Up,
            Direc::Left => Direc::Right,
            Direc::Right => Direc::Left,
        }
    }
}

impl Distribution<Direc> for Standard {
    fn sample<R>(&self, rng: &mut R) -> Direc
    where
        R: Rng + ?Sized,
    {
        let arr = [Direc::Up, Direc::Left, Direc::Down, Direc::Right];
        arr[rng.gen::<u8>() as usize & 0x3]
    }
}

/// Iterator over directions.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DirecIter {
    curr: Option<Direc>,
}

impl Iterator for DirecIter {
    type Item = Direc;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr?;
        self.curr = match curr {
            Direc::Up => Some(Direc::Left),
            Direc::Left => Some(Direc::Down),
            Direc::Down => Some(Direc::Right),
            Direc::Right => None,
        };
        Some(curr)
    }
}

/// A map from directions to a generic type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirecMap<T> {
    /// Mapped to the up direction.
    pub up: T,
    /// Mapped to the left direction.
    pub left: T,
    /// Mapped to the down direction.
    pub down: T,
    /// Mapped to the right direction.
    pub right: T,
}

impl<T> Index<Direc> for DirecMap<T> {
    type Output = T;

    fn index(&self, index: Direc) -> &Self::Output {
        match index {
            Direc::Up => &self.up,
            Direc::Left => &self.left,
            Direc::Down => &self.down,
            Direc::Right => &self.right,
        }
    }
}

impl<T> IndexMut<Direc> for DirecMap<T> {
    fn index_mut(&mut self, index: Direc) -> &mut Self::Output {
        match index {
            Direc::Up => &mut self.up,
            Direc::Left => &mut self.left,
            Direc::Down => &mut self.down,
            Direc::Right => &mut self.right,
        }
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
    pub start: Coord2<Nat>,
    /// The size of this rectangle.
    pub size: Coord2<Nat>,
}

impl Rect {
    /// Calculates and returns the end point (bottom-right) of this rectangle.
    pub fn end(self) -> Coord2<Nat> {
        self.start.zip_with(self.size, |start, size| start + size)
    }

    /// Tests if a point is inside the rectangle.
    pub fn has_point(self, point: Coord2<Nat>) -> bool {
        Axis::iter().all(|axis| {
            point[axis] >= self.start[axis] && point[axis] < self.end()[axis]
        })
    }

    /// Tests if the rectangles are overlapping.
    pub fn overlaps(self, other: Self) -> bool {
        Axis::iter().all(|axis| {
            self.start[axis] <= other.end()[axis]
                && other.start[axis] <= self.end()[axis]
        })
    }

    /// Returns the overlapped area of the given rectangles.
    pub fn overlapped(self, other: Self) -> Option<Rect> {
        let start = self.start.zip_with(other.start, Ord::max);
        let end = self.end().zip_with(other.end(), Ord::min);
        let size = end.zip_with(start, Nat::checked_sub).transpose()?;
        Some(Rect { start, size })
    }

    /// Tests if self moving from the origin crashes on other.
    pub fn moves_through(self, other: Self, origin: Nat, axis: Axis) -> bool {
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

    /// Iterator over columns. This is equivalent to a double for such that: Y
    /// axis is in the inner loop, X in the outer loop.
    pub fn columns(self) -> RectColumns {
        RectColumns { rect: self, curr: self.start }
    }

    /// Iterator over lines. This is equivalent to a double for such that: X
    /// axis is in the inner loop, Y in the outer loop.
    pub fn lines(self) -> RectLines {
        RectLines { rect: self, curr: self.start }
    }

    /// Iterator over the inner borders of the rectangle.
    pub fn borders(self) -> RectBorders {
        RectBorders { rect: self, fixed_axis: Axis::X, curr: self.start }
    }
}

/// Iterator over the inner borders of a rectangle.
#[derive(Debug)]
pub struct RectBorders {
    rect: Rect,
    fixed_axis: Axis,
    curr: Coord2<Nat>,
}

impl Iterator for RectBorders {
    type Item = Coord2<Nat>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.fixed_axis {
            Axis::X => {
                if self.curr.y >= self.rect.end().y {
                    if self.curr.x >= self.rect.end().x - 1 {
                        self.curr.y = self.rect.start.y;
                        self.curr.x = self.rect.start.x + 1;
                        self.fixed_axis = Axis::Y;
                    } else {
                        self.curr.x = self.rect.end().x - 1;
                        self.curr.y = self.rect.start.y;
                    }
                }
            },
            Axis::Y => {
                if self.curr.x >= self.rect.end().x - 1 {
                    if self.curr.y >= self.rect.end().y - 1 {
                        return None;
                    }
                    self.curr.y = self.rect.end().y - 1;
                    self.curr.x = self.rect.start.x + 1;
                }
            },
        }

        let curr = self.curr;
        self.curr[!self.fixed_axis] += 1;
        Some(curr)
    }
}

/// Iterator over columns. This is equivalent to a double for such that: Y
/// axis is in the inner loop, X in the outer loop.
#[derive(Debug)]
pub struct RectColumns {
    rect: Rect,
    curr: Coord2<Nat>,
}

impl Iterator for RectColumns {
    type Item = Coord2<Nat>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.y == self.rect.end().y {
            self.curr.x += 1;

            if self.curr.x >= self.rect.end().x {
                return None;
            }

            self.curr.y = self.rect.start.y;
        }
        let curr = self.curr;
        self.curr.y += 1;
        Some(curr)
    }
}

/// Iterator over lines. This is equivalent to a double for such that: X
/// axis is in the inner loop, Y in the outer loop.
#[derive(Debug)]
pub struct RectLines {
    rect: Rect,
    curr: Coord2<Nat>,
}

impl Iterator for RectLines {
    type Item = Coord2<Nat>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.x == self.rect.end().x {
            self.curr.y += 1;

            if self.curr.y >= self.rect.end().y {
                return None;
            }
            self.curr.x = self.rect.start.x;
        }
        let curr = self.curr;
        self.curr.x += 1;
        Some(curr)
    }
}

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
/// Coordinates of where the game Camera is showing.
pub struct Camera {
    /// Crop of the screen that the player sees.
    rect: Rect,
    offset: Coord2<Nat>,
}

impl Camera {
    /// Builds a new Camera from a position approximately in the center and the
    /// available size.
    pub fn new(
        center: Coord2<Nat>,
        screen_size: Coord2<Nat>,
        offset: Coord2<Nat>,
    ) -> Self {
        Self {
            rect: Rect {
                start: center.zip_with(screen_size, |center, screen_size| {
                    center.saturating_sub(screen_size / 2)
                }),
                size: screen_size,
            },
            offset,
        }
    }

    #[inline]
    /// Returns the crop of this camera.
    pub fn rect(self) -> Rect {
        self.rect
    }

    #[inline]
    /// Returns the screen offset of this camera.
    pub fn offset(self) -> Coord2<Nat> {
        self.offset
    }

    /// Updates the camera to follow the center of the player with at least the
    /// given distance from the center to the edges.
    pub fn update(
        &mut self,
        direc: Direc,
        center: Coord2<Nat>,
        dist: Nat,
    ) -> bool {
        match direc {
            Direc::Up => {
                let diff = center.y.checked_sub(self.rect.start.y);
                if diff.filter(|&y| y >= dist).is_none() {
                    self.rect.start.y = center.y.saturating_sub(dist);
                    true
                } else {
                    false
                }
            },

            Direc::Down => {
                let diff = self.rect.end().y.checked_sub(center.y + 1);
                if diff.filter(|&y| y >= dist).is_none() {
                    self.rect.start.y =
                        (center.y - self.rect.size.y).saturating_add(dist + 1);
                    true
                } else {
                    false
                }
            },

            Direc::Left => {
                let diff = center.x.checked_sub(self.rect.start.x);
                if diff.filter(|&x| x >= dist).is_none() {
                    self.rect.start.x = center.x.saturating_sub(dist);
                    true
                } else {
                    false
                }
            },

            Direc::Right => {
                let diff = self.rect.end().x.checked_sub(center.x + 1);
                if diff.filter(|&x| x >= dist).is_none() {
                    self.rect.start.x =
                        (center.x - self.rect.size.x).saturating_add(dist + 1);
                    true
                } else {
                    false
                }
            },
        }
    }

    /// Converts an absolute point in the map to a point in the screen.
    pub fn convert(self, point: Coord2<Nat>) -> Option<Coord2<Nat>> {
        if self.rect.has_point(point) {
            Some(
                point
                    .zip_with(self.rect.start, Sub::sub)
                    .zip_with(self.offset, Add::add),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Axis, Coord2, Rect};

    #[test]
    fn axis_iter() {
        let vec = Axis::iter().collect::<Vec<_>>();
        assert_eq!(vec, &[Axis::X, Axis::Y]);
    }

    #[test]
    fn overlaps() {
        let rect1 =
            Rect { start: Coord2 { x: 0, y: 0 }, size: Coord2 { x: 8, y: 20 } };
        let rect2 =
            Rect { start: Coord2 { x: 2, y: 16 }, size: Coord2 { x: 6, y: 8 } };
        let rect3 = Rect {
            start: Coord2 { x: 20, y: 16 },
            size: Coord2 { x: 6, y: 8 },
        };
        assert!(rect1.overlaps(rect2));
        assert!(rect2.overlaps(rect1));
        assert!(!rect1.overlaps(rect3));
        assert!(!rect3.overlaps(rect1));
        assert!(!rect2.overlaps(rect3));
        assert!(!rect3.overlaps(rect2));
    }

    #[test]
    fn overlapped_area() {
        let rect1 =
            Rect { start: Coord2 { x: 0, y: 0 }, size: Coord2 { x: 5, y: 5 } };

        let rect2 =
            Rect { start: Coord2 { x: 6, y: 0 }, size: Coord2 { x: 5, y: 5 } };

        let rect3 =
            Rect { start: Coord2 { x: 5, y: 0 }, size: Coord2 { x: 5, y: 5 } };

        let rect4 =
            Rect { start: Coord2 { x: 1, y: 1 }, size: Coord2 { x: 3, y: 3 } };

        assert_eq!(rect1.overlapped(rect2), None);
        assert_eq!(
            rect1.overlapped(rect3),
            Some(Rect {
                start: Coord2 { x: 5, y: 0 },
                size: Coord2 { x: 0, y: 5 }
            })
        );
        assert_eq!(rect1.overlapped(rect4), Some(rect4));
    }
}
