use std::{
    fmt,
    ops::{Add, Index, IndexMut, Sub},
};

use num::{
    traits::{SaturatingAdd, SaturatingSub},
    CheckedAdd,
    CheckedSub,
    One,
};

use crate::CoordPair;

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
pub enum Order {
    Backwards,
    Forwards,
}

impl Order {
    pub const ALL: [Self; 2] = [Self::Backwards, Self::Forwards];

    pub fn move_unit<C>(self, target: C) -> C
    where
        C: Add<Output = C> + Sub<Output = C> + One,
    {
        self.move_by(C::one(), target)
    }

    pub fn checked_move_unit<C>(self, target: &C) -> Option<C>
    where
        C: CheckedAdd + CheckedSub + One,
    {
        self.checked_move_by(&C::one(), target)
    }

    pub fn saturating_move_unit<C>(self, target: &C) -> C
    where
        C: SaturatingAdd + SaturatingSub + One,
    {
        self.saturating_move_by(&C::one(), target)
    }

    pub fn move_by<C>(self, magnitude: C, target: C) -> C
    where
        C: Add<Output = C> + Sub<Output = C>,
    {
        match self {
            Self::Backwards => target - magnitude,
            Self::Forwards => target + magnitude,
        }
    }

    pub fn checked_move_by<C>(self, magnitude: &C, target: &C) -> Option<C>
    where
        C: CheckedAdd + CheckedSub,
    {
        match self {
            Self::Backwards => target.checked_sub(magnitude),
            Self::Forwards => target.checked_add(magnitude),
        }
    }

    pub fn saturating_move_by<C>(self, magnitude: &C, target: &C) -> C
    where
        C: SaturatingAdd + SaturatingSub,
    {
        match self {
            Self::Backwards => target.saturating_add(magnitude),
            Self::Forwards => target.saturating_sub(magnitude),
        }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backwards => write!(f, "-"),
            Self::Forwards => write!(f, "+"),
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

    pub fn new(axis: Axis, order: Order) -> Self {
        match (axis, order) {
            (Axis::Y, Order::Backwards) => Self::Up,
            (Axis::X, Order::Backwards) => Self::Left,
            (Axis::Y, Order::Forwards) => Self::Down,
            (Axis::X, Order::Forwards) => Self::Right,
        }
    }

    pub fn axis(self) -> Axis {
        match self {
            Self::Up | Self::Down => Axis::Y,
            Self::Left | Self::Right => Axis::X,
        }
    }

    pub fn order(self) -> Order {
        match self {
            Self::Up | Self::Left => Order::Backwards,
            Self::Down | Self::Right => Order::Forwards,
        }
    }

    pub fn move_unit<C>(self, target: CoordPair<C>) -> CoordPair<C>
    where
        C: Add<Output = C> + Sub<Output = C> + One,
    {
        DirectionVec::unit(self).mov(target)
    }

    pub fn checked_move_unit<C>(
        self,
        target: &CoordPair<C>,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + One + Clone,
    {
        DirectionVec::unit(self).by_ref().checked_move(target)
    }

    pub fn checked_move_unit_by_ref<C>(
        self,
        target: CoordPair<&C>,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + One + Clone,
    {
        DirectionVec::unit(self).by_ref().checked_move_by_ref(target)
    }

    pub fn saturating_move_unit<C>(self, target: &CoordPair<C>) -> CoordPair<C>
    where
        C: SaturatingAdd + SaturatingSub + One + Clone,
    {
        DirectionVec::unit(self).by_ref().saturating_move(target)
    }

    pub fn saturating_move_unit_by_ref<C>(
        self,
        target: CoordPair<&C>,
    ) -> CoordPair<C>
    where
        C: SaturatingAdd + SaturatingSub + One + Clone,
    {
        DirectionVec::unit(self).by_ref().saturating_move_by_ref(target)
    }

    pub fn move_by<C>(self, magnitude: C, target: CoordPair<C>) -> CoordPair<C>
    where
        C: Add<Output = C> + Sub<Output = C>,
    {
        DirectionVec { direction: self, magnitude }.mov(target)
    }

    pub fn checked_move_by<C>(
        self,
        magnitude: &C,
        target: &CoordPair<C>,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + Clone,
    {
        DirectionVec { direction: self, magnitude }.checked_move(target)
    }

    pub fn checked_move_by_ref_by<C>(
        self,
        magnitude: &C,
        target: CoordPair<&C>,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + Clone,
    {
        DirectionVec { direction: self, magnitude }.checked_move_by_ref(target)
    }

    pub fn saturating_move_by<C>(
        self,
        magnitude: &C,
        target: &CoordPair<C>,
    ) -> CoordPair<C>
    where
        C: SaturatingAdd + SaturatingSub + Clone,
    {
        self.saturating_move_by_ref_by(magnitude, target.as_ref())
    }

    pub fn saturating_move_by_ref_by<C>(
        self,
        magnitude: &C,
        target: CoordPair<&C>,
    ) -> CoordPair<C>
    where
        C: SaturatingAdd + SaturatingSub + Clone,
    {
        match self.axis() {
            Axis::Y => CoordPair {
                y: self.order().saturating_move_by(magnitude, target.y),
                x: target.x.clone(),
            },
            Axis::X => CoordPair {
                x: self.order().saturating_move_by(magnitude, target.x),
                y: target.y.clone(),
            },
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DirectionMap<T> {
    pub up: T,
    pub left: T,
    pub down: T,
    pub right: T,
}

impl<T> DirectionMap<T> {
    pub fn from_dirs<F>(mut generator: F) -> DirectionMap<T>
    where
        F: FnMut(Direction) -> T,
    {
        Self {
            up: generator(Direction::Up),
            left: generator(Direction::Left),
            down: generator(Direction::Down),
            right: generator(Direction::Right),
        }
    }

    pub fn map<F, U>(self, mut mapper: F) -> DirectionMap<U>
    where
        F: FnMut(T) -> U,
    {
        self.map_with_dirs(|elem, _| mapper(elem))
    }

    pub fn map_with_dirs<F, U>(self, mut mapper: F) -> DirectionMap<U>
    where
        F: FnMut(T, Direction) -> U,
    {
        DirectionMap {
            up: mapper(self.up, Direction::Up),
            left: mapper(self.left, Direction::Left),
            down: mapper(self.down, Direction::Down),
            right: mapper(self.right, Direction::Right),
        }
    }

    pub fn as_ref(&self) -> DirectionMap<&T> {
        DirectionMap {
            up: &self.up,
            left: &self.left,
            down: &self.down,
            right: &self.right,
        }
    }

    pub fn as_mut(&mut self) -> DirectionMap<&mut T> {
        DirectionMap {
            up: &mut self.up,
            left: &mut self.left,
            down: &mut self.down,
            right: &mut self.right,
        }
    }
}

impl<'a, T> DirectionMap<&'a T> {
    pub fn copied(self) -> DirectionMap<T>
    where
        T: Copy,
    {
        self.map(|a| *a)
    }

    pub fn cloned(self) -> DirectionMap<T>
    where
        T: Clone,
    {
        self.map(Clone::clone)
    }
}

impl<'a, T> DirectionMap<&'a mut T> {
    pub fn copied(self) -> DirectionMap<T>
    where
        T: Copy,
    {
        self.map(|a| *a)
    }

    pub fn cloned(self) -> DirectionMap<T>
    where
        T: Clone,
    {
        self.map(|a| a.clone())
    }

    pub fn share(self) -> DirectionMap<&'a T> {
        self.map(|a| &*a)
    }
}

impl<T> DirectionMap<Option<T>> {
    pub fn transpose(self) -> Option<DirectionMap<T>> {
        Some(DirectionMap {
            up: self.up?,
            left: self.left?,
            down: self.down?,
            right: self.right?,
        })
    }

    pub fn from_transposed(transposed: Option<DirectionMap<T>>) -> Self {
        match transposed {
            Some(table) => table.map(Some),
            None => Self::from_dirs(|_| None),
        }
    }
}

impl<T, E> DirectionMap<Result<T, E>> {
    pub fn transpose(self) -> Result<DirectionMap<T>, E> {
        Ok(DirectionMap {
            up: self.up?,
            left: self.left?,
            down: self.down?,
            right: self.right?,
        })
    }

    pub fn from_transposed(transposed: Result<DirectionMap<T>, E>) -> Self
    where
        E: Clone,
    {
        match transposed {
            Ok(table) => table.map(Ok),
            Err(error) => Self::from_dirs(|_| Err(error.clone())),
        }
    }
}

impl<T> Index<Direction> for DirectionMap<T> {
    type Output = T;

    fn index(&self, index: Direction) -> &Self::Output {
        match index {
            Direction::Up => &self.up,
            Direction::Left => &self.left,
            Direction::Down => &self.down,
            Direction::Right => &self.right,
        }
    }
}

impl<T> Index<(Axis, Order)> for DirectionMap<T> {
    type Output = T;

    fn index(&self, (axis, order): (Axis, Order)) -> &Self::Output {
        &self[Direction::new(axis, order)]
    }
}

impl<T> IndexMut<Direction> for DirectionMap<T> {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        match index {
            Direction::Up => &mut self.up,
            Direction::Left => &mut self.left,
            Direction::Down => &mut self.down,
            Direction::Right => &mut self.right,
        }
    }
}

impl<T> IndexMut<(Axis, Order)> for DirectionMap<T> {
    fn index_mut(&mut self, (axis, order): (Axis, Order)) -> &mut Self::Output {
        &mut self[Direction::new(axis, order)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirectionVec<C> {
    pub direction: Direction,
    pub magnitude: C,
}

impl<C> DirectionVec<C> {
    pub fn unit(direction: Direction) -> Self
    where
        C: One,
    {
        Self { direction, magnitude: C::one() }
    }

    pub fn by_ref(&self) -> DirectionVec<&C> {
        DirectionVec { direction: self.direction, magnitude: &self.magnitude }
    }

    pub fn mov(self, target: CoordPair<C>) -> CoordPair<C>
    where
        C: Add<Output = C> + Sub<Output = C>,
    {
        match self.direction.axis() {
            Axis::Y => CoordPair {
                y: self.direction.order().move_by(self.magnitude, target.y),
                ..target
            },
            Axis::X => CoordPair {
                x: self.direction.order().move_by(self.magnitude, target.x),
                ..target
            },
        }
    }
}

impl<'a, C> DirectionVec<&'a C> {
    pub fn checked_move(self, target: &CoordPair<C>) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + Clone,
    {
        self.checked_move_by_ref(target.as_ref())
    }

    pub fn checked_move_by_ref(
        self,
        target: CoordPair<&C>,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + Clone,
    {
        match self.direction.axis() {
            Axis::Y => Some(CoordPair {
                y: self
                    .direction
                    .order()
                    .checked_move_by(self.magnitude, target.y)?,
                x: target.x.clone(),
            }),
            Axis::X => Some(CoordPair {
                x: self
                    .direction
                    .order()
                    .checked_move_by(self.magnitude, target.x)?,
                y: target.y.clone(),
            }),
        }
    }

    pub fn saturating_move(self, target: &CoordPair<C>) -> CoordPair<C>
    where
        C: SaturatingAdd + SaturatingSub + Clone,
    {
        self.saturating_move_by_ref(target.as_ref())
    }

    pub fn saturating_move_by_ref(self, target: CoordPair<&C>) -> CoordPair<C>
    where
        C: SaturatingAdd + SaturatingSub + Clone,
    {
        match self.direction.axis() {
            Axis::Y => CoordPair {
                y: self
                    .direction
                    .order()
                    .saturating_move_by(self.magnitude, target.y),
                x: target.x.clone(),
            },
            Axis::X => CoordPair {
                x: self
                    .direction
                    .order()
                    .saturating_move_by(self.magnitude, target.x),
                y: target.y.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Diagonal {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Diagonal {
    pub const ALL: [Self; 4] =
        [Self::TopLeft, Self::TopRight, Self::BottomLeft, Self::BottomRight];

    pub fn new(axes_orders: CoordPair<Order>) -> Self {
        match axes_orders {
            CoordPair { y: Order::Backwards, x: Order::Backwards } => {
                Self::TopLeft
            },
            CoordPair { y: Order::Backwards, x: Order::Forwards } => {
                Self::TopRight
            },
            CoordPair { y: Order::Forwards, x: Order::Backwards } => {
                Self::BottomLeft
            },
            CoordPair { y: Order::Forwards, x: Order::Forwards } => {
                Self::BottomRight
            },
        }
    }

    pub fn axes_orders(self) -> CoordPair<Order> {
        match self {
            Self::TopLeft => {
                CoordPair { y: Order::Backwards, x: Order::Backwards }
            },
            Self::TopRight => {
                CoordPair { y: Order::Backwards, x: Order::Forwards }
            },
            Self::BottomLeft => {
                CoordPair { y: Order::Forwards, x: Order::Backwards }
            },
            Self::BottomRight => {
                CoordPair { y: Order::Forwards, x: Order::Forwards }
            },
        }
    }
}

impl fmt::Display for Diagonal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TopLeft => write!(f, "-y -x"),
            Self::TopRight => write!(f, "-y +x"),
            Self::BottomLeft => write!(f, "+y -x"),
            Self::BottomRight => write!(f, "+y +x"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DiagonalMap<T> {
    pub top_left: T,
    pub top_right: T,
    pub bottom_left: T,
    pub bottom_right: T,
}

impl<T> Index<Diagonal> for DiagonalMap<T> {
    type Output = T;

    fn index(&self, index: Diagonal) -> &Self::Output {
        match index {
            Diagonal::TopLeft => &self.top_left,
            Diagonal::TopRight => &self.top_right,
            Diagonal::BottomLeft => &self.bottom_left,
            Diagonal::BottomRight => &self.bottom_right,
        }
    }
}

impl<T> Index<CoordPair<Order>> for DiagonalMap<T> {
    type Output = T;

    fn index(&self, index: CoordPair<Order>) -> &Self::Output {
        &self[Diagonal::new(index)]
    }
}

impl<T> IndexMut<Diagonal> for DiagonalMap<T> {
    fn index_mut(&mut self, index: Diagonal) -> &mut Self::Output {
        match index {
            Diagonal::TopLeft => &mut self.top_left,
            Diagonal::TopRight => &mut self.top_right,
            Diagonal::BottomLeft => &mut self.bottom_left,
            Diagonal::BottomRight => &mut self.bottom_right,
        }
    }
}

impl<T> IndexMut<CoordPair<Order>> for DiagonalMap<T> {
    fn index_mut(&mut self, index: CoordPair<Order>) -> &mut Self::Output {
        &mut self[Diagonal::new(index)]
    }
}
