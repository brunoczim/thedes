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
        self.move_by(C::one(), target)
    }

    pub fn checked_move_unit<C>(
        self,
        target: &CoordPair<C>,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + One + Clone,
    {
        self.checked_move_by(&C::one(), target)
    }

    pub fn checked_move_unit_ref_by<C>(
        self,
        target: CoordPair<&C>,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + One + Clone,
    {
        self.checked_move_by_ref_by(&C::one(), target)
    }

    pub fn saturating_move_unit<C>(self, target: &CoordPair<C>) -> CoordPair<C>
    where
        C: SaturatingAdd + SaturatingSub + One + Clone,
    {
        self.saturating_move_by(&C::one(), target)
    }

    pub fn saturating_move_unit_by_ref<C>(
        self,
        target: CoordPair<&C>,
    ) -> CoordPair<C>
    where
        C: SaturatingAdd + SaturatingSub + One + Clone,
    {
        self.saturating_move_by_ref_by(&C::one(), target)
    }

    pub fn move_by<C>(self, magnitude: C, target: CoordPair<C>) -> CoordPair<C>
    where
        C: Add<Output = C> + Sub<Output = C>,
    {
        match self.axis() {
            Axis::Y => CoordPair {
                y: self.order().move_by(magnitude, target.y),
                ..target
            },
            Axis::X => CoordPair {
                x: self.order().move_by(magnitude, target.x),
                ..target
            },
        }
    }

    pub fn checked_move_by<C>(
        self,
        magnitude: &C,
        target: &CoordPair<C>,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + Clone,
    {
        self.checked_move_by_ref_by(magnitude, target.as_ref())
    }

    pub fn checked_move_by_ref_by<C>(
        self,
        magnitude: &C,
        target: CoordPair<&C>,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + Clone,
    {
        match self.axis() {
            Axis::Y => Some(CoordPair {
                y: self.order().checked_move_by(magnitude, target.y)?,
                x: target.x.clone(),
            }),
            Axis::X => Some(CoordPair {
                x: self.order().checked_move_by(magnitude, target.x)?,
                y: target.y.clone(),
            }),
        }
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
