use std::{
    fmt,
    ops::{
        Add,
        AddAssign,
        Div,
        DivAssign,
        Index,
        IndexMut,
        Mul,
        MulAssign,
        Neg,
        Rem,
        RemAssign,
        Sub,
        SubAssign,
    },
};

use num::{
    traits::CheckedRem,
    CheckedAdd,
    CheckedDiv,
    CheckedMul,
    CheckedSub,
    Integer,
    One,
    Zero,
};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Invalid point {point} for rectangle {rect}")]
pub struct InvalidRectPoint<C>
where
    C: fmt::Display,
{
    pub point: CoordPair<C>,
    pub rect: Rect<C>,
}

#[derive(Debug, Error)]
#[error("Invalid area {area} for partition of rectangle {rect}")]
pub struct InvalidArea<C>
where
    C: fmt::Display,
{
    pub area: C,
    pub rect: Rect<C>,
}

#[derive(Debug, Error)]
pub enum HorzAreaError<C>
where
    C: fmt::Display,
{
    #[error("Point outside of rectangle defines no internal area")]
    InvalidRectPoint(
        #[from]
        #[source]
        InvalidRectPoint<C>,
    ),
    #[error("Arithmetic overflow computing area for rectangle of size {size}")]
    Overflow { size: CoordPair<C> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    Y,
    X,
}

impl Axis {
    pub const ALL: [Self; 2] = [Self::Y, Self::X];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CoordPair<C> {
    pub y: C,
    pub x: C,
}

impl<C> CoordPair<C> {
    pub fn from_axes<F>(mut generator: F) -> Self
    where
        F: FnMut(Axis) -> C,
    {
        Self { y: generator(Axis::Y), x: generator(Axis::X) }
    }

    pub fn try_from_axes<F, E>(mut generator: F) -> Result<Self, E>
    where
        F: FnMut(Axis) -> Result<C, E>,
    {
        Ok(Self { y: generator(Axis::Y)?, x: generator(Axis::X)? })
    }

    pub fn as_ref(&self) -> CoordPair<&C> {
        CoordPair::from_axes(|axis| &self[axis])
    }

    pub fn as_mut(&mut self) -> CoordPair<&mut C> {
        CoordPair { y: &mut self.y, x: &mut self.x }
    }

    pub fn map<F, C0>(self, mut mapper: F) -> CoordPair<C0>
    where
        F: FnMut(C) -> C0,
    {
        CoordPair { y: mapper(self.y), x: mapper(self.x) }
    }

    pub fn try_map<F, C0, E>(self, mut mapper: F) -> Result<CoordPair<C0>, E>
    where
        F: FnMut(C) -> Result<C0, E>,
    {
        Ok(CoordPair { y: mapper(self.y)?, x: mapper(self.x)? })
    }

    pub fn map_with_axes<F, C0>(self, mut mapper: F) -> CoordPair<C0>
    where
        F: FnMut(C, Axis) -> C0,
    {
        CoordPair { y: mapper(self.y, Axis::Y), x: mapper(self.x, Axis::X) }
    }

    pub fn try_map_with_axes<F, C0, E>(
        self,
        mut mapper: F,
    ) -> Result<CoordPair<C0>, E>
    where
        F: FnMut(C, Axis) -> Result<C0, E>,
    {
        Ok(CoordPair {
            y: mapper(self.y, Axis::Y)?,
            x: mapper(self.x, Axis::X)?,
        })
    }

    pub fn zip2<C0>(self, other: CoordPair<C0>) -> CoordPair<(C, C0)> {
        CoordPair { y: (self.y, other.y), x: (self.x, other.x) }
    }

    pub fn zip2_with<F, C0, C1>(
        self,
        other: CoordPair<C0>,
        mut zipper: F,
    ) -> CoordPair<C1>
    where
        F: FnMut(C, C0) -> C1,
    {
        self.zip2(other).map(|(a, b)| zipper(a, b))
    }

    pub fn zip2_with_axes<F, C0, C1>(
        self,
        other: CoordPair<C0>,
        mut zipper: F,
    ) -> CoordPair<C1>
    where
        F: FnMut(C, C0, Axis) -> C1,
    {
        self.zip2(other).map_with_axes(|(a, b), axis| zipper(a, b, axis))
    }

    pub fn try_zip2_with<F, C0, C1, E>(
        self,
        other: CoordPair<C0>,
        mut zipper: F,
    ) -> Result<CoordPair<C1>, E>
    where
        F: FnMut(C, C0) -> Result<C1, E>,
    {
        self.zip2(other).try_map(|(a, b)| zipper(a, b))
    }

    pub fn try_zip2_with_axes<F, C0, C1, E>(
        self,
        other: CoordPair<C0>,
        mut zipper: F,
    ) -> Result<CoordPair<C1>, E>
    where
        F: FnMut(C, C0, Axis) -> Result<C1, E>,
    {
        self.zip2(other).try_map_with_axes(|(a, b), axis| zipper(a, b, axis))
    }

    pub fn zip3<C0, C1>(
        self,
        other: CoordPair<C0>,
        another: CoordPair<C1>,
    ) -> CoordPair<(C, C0, C1)> {
        CoordPair {
            y: (self.y, other.y, another.y),
            x: (self.x, other.x, another.x),
        }
    }

    pub fn zip3_with<F, C0, C1, C2>(
        self,
        other: CoordPair<C0>,
        another: CoordPair<C1>,
        mut zipper: F,
    ) -> CoordPair<C2>
    where
        F: FnMut(C, C0, C1) -> C2,
    {
        self.zip3(other, another).map(|(a, b, c)| zipper(a, b, c))
    }

    pub fn zip3_with_axes<F, C0, C1, C2>(
        self,
        other: CoordPair<C0>,
        another: CoordPair<C1>,
        mut zipper: F,
    ) -> CoordPair<C2>
    where
        F: FnMut(C, C0, C1, Axis) -> C2,
    {
        self.zip3(other, another)
            .map_with_axes(|(a, b, c), axis| zipper(a, b, c, axis))
    }

    pub fn try_zip3_with<F, C0, C1, C2, E>(
        self,
        other: CoordPair<C0>,
        another: CoordPair<C1>,
        mut zipper: F,
    ) -> Result<CoordPair<C2>, E>
    where
        F: FnMut(C, C0, C1) -> Result<C2, E>,
    {
        self.zip3(other, another).try_map(|(a, b, c)| zipper(a, b, c))
    }

    pub fn try_zip3_with_axes<F, C0, C1, C2, E>(
        self,
        other: CoordPair<C0>,
        another: CoordPair<C1>,
        mut zipper: F,
    ) -> Result<CoordPair<C2>, E>
    where
        F: FnMut(C, C0, C1, Axis) -> Result<C2, E>,
    {
        self.zip3(other, another)
            .try_map_with_axes(|(a, b, c), axis| zipper(a, b, c, axis))
    }

    pub fn all<F>(self, mut predicate: F) -> bool
    where
        F: FnMut(C) -> bool,
    {
        predicate(self.y) && predicate(self.x)
    }

    pub fn any<F>(self, mut predicate: F) -> bool
    where
        F: FnMut(C) -> bool,
    {
        predicate(self.y) || predicate(self.x)
    }

    pub fn div_floor_by(&self, divisor: &C) -> Self
    where
        C: Integer,
    {
        self.as_ref().div_floor_by_ref_by(divisor)
    }

    pub fn div_ceil_by(&self, divisor: &C) -> Self
    where
        C: Integer,
    {
        self.as_ref().div_ceil_by_ref_by(divisor)
    }

    pub fn div_floor_on(&self, dividend: &C) -> Self
    where
        C: Integer,
    {
        self.as_ref().div_floor_by_ref_on(dividend)
    }

    pub fn div_ceil_on(&self, dividend: &C) -> Self
    where
        C: Integer,
    {
        self.as_ref().div_ceil_by_ref_on(dividend)
    }

    pub fn div_floor(&self, other: &Self) -> Self
    where
        C: Integer,
    {
        self.as_ref().div_floor_by_ref(other.as_ref())
    }

    pub fn div_ceil(&self, other: &Self) -> Self
    where
        C: Integer,
    {
        self.as_ref().div_ceil_by_ref(other.as_ref())
    }

    pub fn move_unit(self, direction: Direction) -> Self
    where
        C: Add<Output = C> + Sub<Output = C> + One,
    {
        self.move_by(C::one(), direction)
    }

    pub fn checked_move_unit(&self, direction: Direction) -> Option<Self>
    where
        C: CheckedAdd + CheckedSub + One + Clone,
    {
        self.as_ref().checked_move_unit_by_ref(direction)
    }

    pub fn move_by(self, magnitude: C, direction: Direction) -> Self
    where
        C: Add<Output = C> + Sub<Output = C>,
    {
        match direction {
            Direction::Up => Self { x: self.x, y: self.y - magnitude },
            Direction::Left => Self { x: self.x - magnitude, y: self.y },
            Direction::Down => Self { x: self.x, y: self.y + magnitude },
            Direction::Right => Self { x: self.x + magnitude, y: self.y },
        }
    }

    pub fn checked_move_by(
        &self,
        magnitude: &C,
        direction: Direction,
    ) -> Option<Self>
    where
        C: CheckedAdd + CheckedSub + Clone,
    {
        self.as_ref().checked_move_by_ref_by(magnitude, direction)
    }

    pub fn as_rect_size(self, top_left: Self) -> Rect<C> {
        Rect { top_left, size: self }
    }

    pub fn as_rect_top_left(self, size: Self) -> Rect<C> {
        Rect { top_left: self, size }
    }
}

impl<'a, C> CoordPair<&'a C> {
    pub fn copied(self) -> CoordPair<C>
    where
        C: Copy,
    {
        self.map(|a| *a)
    }

    pub fn cloned(self) -> CoordPair<C>
    where
        C: Clone,
    {
        self.map(C::clone)
    }

    pub fn checked_add_by_ref(self, other: Self) -> Option<CoordPair<C>>
    where
        C: CheckedAdd,
    {
        self.zip2_with(other, C::checked_add).transpose()
    }

    pub fn checked_sub_by_ref(self, other: Self) -> Option<CoordPair<C>>
    where
        C: CheckedSub,
    {
        self.zip2_with(other, C::checked_sub).transpose()
    }

    pub fn checked_mul_by_ref(self, other: Self) -> Option<CoordPair<C>>
    where
        C: CheckedMul,
    {
        self.zip2_with(other, C::checked_mul).transpose()
    }

    pub fn checked_div_by_ref(self, other: Self) -> Option<CoordPair<C>>
    where
        C: CheckedDiv,
    {
        self.zip2_with(other, C::checked_div).transpose()
    }

    pub fn checked_rem_by_ref(self, other: Self) -> Option<CoordPair<C>>
    where
        C: CheckedRem,
    {
        self.zip2_with(other, C::checked_rem).transpose()
    }

    pub fn div_floor_by_ref_by(self, divisor: &C) -> CoordPair<C>
    where
        C: Integer,
    {
        self.map(|dividend| dividend.div_floor(divisor))
    }

    pub fn div_ceil_by_ref_by(self, divisor: &C) -> CoordPair<C>
    where
        C: Integer,
    {
        self.map(|dividend| dividend.div_ceil(divisor))
    }

    pub fn div_floor_by_ref_on(self, dividend: &C) -> CoordPair<C>
    where
        C: Integer,
    {
        self.map(|divisor| divisor.div_floor(dividend))
    }

    pub fn div_ceil_by_ref_on(self, dividend: &C) -> CoordPair<C>
    where
        C: Integer,
    {
        self.map(|divisor| divisor.div_ceil(dividend))
    }

    pub fn div_floor_by_ref(self, other: Self) -> CoordPair<C>
    where
        C: Integer,
    {
        self.zip2_with(other, C::div_floor)
    }

    pub fn div_ceil_by_ref(self, other: Self) -> CoordPair<C>
    where
        C: Integer,
    {
        self.zip2_with(other, C::div_ceil)
    }

    pub fn checked_move_unit_by_ref(
        self,
        direction: Direction,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + One + Clone,
    {
        self.checked_move_by_ref_by(&C::one(), direction)
    }

    pub fn checked_move_by_ref_by(
        self,
        magnitude: &C,
        direction: Direction,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + Clone,
    {
        Some(match direction {
            Direction::Up => CoordPair {
                x: (*self.x).clone(),
                y: self.y.checked_sub(magnitude)?,
            },
            Direction::Left => CoordPair {
                x: self.x.checked_sub(magnitude)?,
                y: (*self.y).clone(),
            },
            Direction::Down => CoordPair {
                x: (*self.x).clone(),
                y: self.y.checked_add(magnitude)?,
            },
            Direction::Right => CoordPair {
                x: self.x.checked_add(magnitude)?,
                y: (*self.y).clone(),
            },
        })
    }
}

impl<'a, C> CoordPair<&'a mut C> {
    pub fn copied(self) -> CoordPair<C>
    where
        C: Copy,
    {
        self.map(|a| *a)
    }

    pub fn cloned(self) -> CoordPair<C>
    where
        C: Clone,
    {
        self.map(|a| a.clone())
    }

    pub fn share(self) -> CoordPair<&'a C> {
        self.map(|a| &*a)
    }
}

impl<C> CoordPair<Option<C>> {
    pub fn transpose(self) -> Option<CoordPair<C>> {
        Some(CoordPair { y: self.y?, x: self.x? })
    }

    pub fn from_transposed(transposed: Option<CoordPair<C>>) -> Self {
        match transposed {
            Some(pair) => pair.map(Some),
            None => Self::from_axes(|_| None),
        }
    }
}

impl<C, E> CoordPair<Result<C, E>> {
    pub fn transpose(self) -> Result<CoordPair<C>, E> {
        Ok(CoordPair { y: self.y?, x: self.x? })
    }

    pub fn from_transposed(transposed: Result<CoordPair<C>, E>) -> Self
    where
        E: Clone,
    {
        match transposed {
            Ok(pair) => pair.map(Ok),
            Err(error) => Self { y: Err(error.clone()), x: Err(error) },
        }
    }
}

impl<C> Index<Axis> for CoordPair<C> {
    type Output = C;

    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::Y => &self.y,
            Axis::X => &self.x,
        }
    }
}

impl<C> IndexMut<Axis> for CoordPair<C> {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        match index {
            Axis::Y => &mut self.y,
            Axis::X => &mut self.x,
        }
    }
}

impl<C> fmt::Display for CoordPair<C>
where
    C: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(x={}, y={})", self.x, self.y)
    }
}

impl<C> Add for CoordPair<C>
where
    C: Add,
{
    type Output = CoordPair<C::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        self.zip2_with(rhs, |a, b| a + b)
    }
}

impl<C> Add<C> for CoordPair<C>
where
    C: Add + Clone,
{
    type Output = CoordPair<C::Output>;

    fn add(self, rhs: C) -> Self::Output {
        CoordPair { y: self.y + rhs.clone(), x: self.x + rhs }
    }
}

impl<C> AddAssign for CoordPair<C>
where
    C: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.as_mut().zip2_with(rhs, |a, b| *a += b);
    }
}

impl<C> AddAssign<C> for CoordPair<C>
where
    C: AddAssign + Clone,
{
    fn add_assign(&mut self, rhs: C) {
        self.x += rhs.clone();
        self.y += rhs;
    }
}

impl<C> Sub for CoordPair<C>
where
    C: Sub,
{
    type Output = CoordPair<C::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.zip2_with(rhs, |a, b| a - b)
    }
}

impl<C> Sub<C> for CoordPair<C>
where
    C: Sub + Clone,
{
    type Output = CoordPair<C::Output>;

    fn sub(self, rhs: C) -> Self::Output {
        CoordPair { y: self.y - rhs.clone(), x: self.x - rhs }
    }
}

impl<C> SubAssign for CoordPair<C>
where
    C: SubAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.as_mut().zip2_with(rhs, |a, b| *a -= b);
    }
}

impl<C> SubAssign<C> for CoordPair<C>
where
    C: SubAssign + Clone,
{
    fn sub_assign(&mut self, rhs: C) {
        self.x -= rhs.clone();
        self.y -= rhs;
    }
}

impl<C> Mul for CoordPair<C>
where
    C: Mul,
{
    type Output = CoordPair<C::Output>;

    fn mul(self, rhs: Self) -> Self::Output {
        self.zip2_with(rhs, |a, b| a * b)
    }
}

impl<C> Mul<C> for CoordPair<C>
where
    C: Mul + Clone,
{
    type Output = CoordPair<C::Output>;

    fn mul(self, rhs: C) -> Self::Output {
        CoordPair { y: self.y * rhs.clone(), x: self.x * rhs }
    }
}

impl<C> MulAssign for CoordPair<C>
where
    C: MulAssign,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.as_mut().zip2_with(rhs, |a, b| *a *= b);
    }
}

impl<C> MulAssign<C> for CoordPair<C>
where
    C: MulAssign + Clone,
{
    fn mul_assign(&mut self, rhs: C) {
        self.x *= rhs.clone();
        self.y *= rhs;
    }
}

impl<C> Div for CoordPair<C>
where
    C: Div,
{
    type Output = CoordPair<C::Output>;

    fn div(self, rhs: Self) -> Self::Output {
        self.zip2_with(rhs, |a, b| a / b)
    }
}

impl<C> Div<C> for CoordPair<C>
where
    C: Div + Clone,
{
    type Output = CoordPair<C::Output>;

    fn div(self, rhs: C) -> Self::Output {
        CoordPair { y: self.y / rhs.clone(), x: self.x / rhs }
    }
}

impl<C> DivAssign for CoordPair<C>
where
    C: DivAssign,
{
    fn div_assign(&mut self, rhs: Self) {
        self.as_mut().zip2_with(rhs, |a, b| *a /= b);
    }
}

impl<C> DivAssign<C> for CoordPair<C>
where
    C: DivAssign + Clone,
{
    fn div_assign(&mut self, rhs: C) {
        self.x /= rhs.clone();
        self.y /= rhs;
    }
}

impl<C> Rem for CoordPair<C>
where
    C: Rem,
{
    type Output = CoordPair<C::Output>;

    fn rem(self, rhs: Self) -> Self::Output {
        self.zip2_with(rhs, |a, b| a % b)
    }
}

impl<C> Rem<C> for CoordPair<C>
where
    C: Rem + Clone,
{
    type Output = CoordPair<C::Output>;

    fn rem(self, rhs: C) -> Self::Output {
        CoordPair { y: self.y % rhs.clone(), x: self.x % rhs }
    }
}

impl<C> RemAssign for CoordPair<C>
where
    C: RemAssign,
{
    fn rem_assign(&mut self, rhs: Self) {
        self.as_mut().zip2_with(rhs, |a, b| *a %= b);
    }
}

impl<C> RemAssign<C> for CoordPair<C>
where
    C: RemAssign + Clone,
{
    fn rem_assign(&mut self, rhs: C) {
        self.x %= rhs.clone();
        self.y %= rhs;
    }
}

impl<C> Zero for CoordPair<C>
where
    C: Zero + PartialEq,
{
    fn zero() -> Self {
        Self::from_axes(|_| C::zero())
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl<C> One for CoordPair<C>
where
    C: One,
{
    fn one() -> Self {
        Self::from_axes(|_| C::one())
    }
}

impl<C> Neg for CoordPair<C>
where
    C: Neg,
{
    type Output = CoordPair<C::Output>;

    fn neg(self) -> Self::Output {
        self.map(|a| -a)
    }
}

impl<C> CheckedAdd for CoordPair<C>
where
    C: CheckedAdd,
{
    fn checked_add(&self, other: &Self) -> Option<Self> {
        self.as_ref().checked_add_by_ref(other.as_ref())
    }
}

impl<C> CheckedSub for CoordPair<C>
where
    C: CheckedSub,
{
    fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.as_ref().checked_sub_by_ref(other.as_ref())
    }
}

impl<C> CheckedMul for CoordPair<C>
where
    C: CheckedMul,
{
    fn checked_mul(&self, other: &Self) -> Option<Self> {
        self.as_ref().checked_mul_by_ref(other.as_ref())
    }
}

impl<C> CheckedDiv for CoordPair<C>
where
    C: CheckedDiv,
{
    fn checked_div(&self, other: &Self) -> Option<Self> {
        self.as_ref().checked_div_by_ref(other.as_ref())
    }
}

impl<C> CheckedRem for CoordPair<C>
where
    C: CheckedRem,
{
    fn checked_rem(&self, other: &Self) -> Option<Self> {
        self.as_ref().checked_rem_by_ref(other.as_ref())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rect<C> {
    pub top_left: CoordPair<C>,
    pub size: CoordPair<C>,
}

impl<C> Rect<C> {
    pub fn as_ref(&self) -> Rect<&C> {
        Rect { top_left: self.top_left.as_ref(), size: self.size.as_ref() }
    }

    pub fn as_mut(&mut self) -> Rect<&mut C> {
        Rect { top_left: self.top_left.as_mut(), size: self.size.as_mut() }
    }

    pub fn map<F, C0>(self, mut mapper: F) -> Rect<C0>
    where
        F: FnMut(C) -> C0,
    {
        Rect {
            top_left: self.top_left.map(&mut mapper),
            size: self.size.map(mapper),
        }
    }

    pub fn try_map<F, C0, E>(self, mut mapper: F) -> Result<Rect<C0>, E>
    where
        F: FnMut(C) -> Result<C0, E>,
    {
        Ok(Rect {
            top_left: self.top_left.try_map(&mut mapper)?,
            size: self.size.try_map(mapper)?,
        })
    }

    pub fn bottom_right<D>(self) -> CoordPair<D>
    where
        C: Add<Output = D>,
    {
        self.top_left.zip2_with(self.size, C::add)
    }

    pub fn checked_bottom_right(&self) -> Option<CoordPair<C>>
    where
        C: CheckedAdd,
    {
        self.top_left.checked_add(&self.size)
    }

    pub fn contains_point(self, point: CoordPair<C>) -> bool
    where
        C: Sub<Output = C> + PartialOrd,
    {
        self.top_left
            .zip3(self.size, point)
            .all(|(start, size, coord)| start >= coord && coord - start < size)
    }

    pub fn checked_horz_area_up_to(
        self,
        point: CoordPair<C>,
    ) -> Result<C, HorzAreaError<C>>
    where
        C: Sub<Output = C> + CheckedAdd + CheckedMul,
        C: fmt::Display + PartialOrd + Clone,
    {
        if self.clone().contains_point(point.clone()) {
            let from_origin = point - self.top_left;
            self.size
                .x
                .checked_mul(&from_origin.y)
                .and_then(|scaled| scaled.checked_add(&from_origin.x))
                .ok_or(HorzAreaError::Overflow { size: self.size })
        } else {
            Err(InvalidRectPoint { point, rect: self })?
        }
    }

    pub fn horz_area_up_to(self, point: CoordPair<C>) -> C
    where
        C: Sub<Output = C> + Mul<Output = C> + Add<Output = C>,
        C: fmt::Display + PartialOrd + Clone,
    {
        let from_origin = point - self.top_left;
        self.size.x * from_origin.y + from_origin.x
    }

    pub fn checked_bot_right_of_horz_area(
        &self,
        area: &C,
    ) -> Result<CoordPair<C>, InvalidArea<C>>
    where
        C: CheckedAdd + CheckedDiv + CheckedRem + Clone,
        C: fmt::Display,
    {
        let optional_coords = CoordPair {
            x: area.checked_rem(&self.size.x),
            y: area.checked_div(&self.size.y),
        };

        optional_coords
            .transpose()
            .and_then(|from_origin| self.top_left.checked_add(&from_origin))
            .ok_or(InvalidArea { area: area.clone(), rect: self.clone() })
    }

    pub fn bot_right_of_horz_area(self, area: C) -> CoordPair<C>
    where
        C: Add<Output = C> + Div<Output = C> + Rem<Output = C> + Clone,
    {
        let x = area.clone() % self.size.x;
        let y = area / self.size.y;
        let from_origin = CoordPair { x, y };
        self.top_left + from_origin
    }

    pub fn total_area<D>(self) -> D
    where
        C: Mul<Output = D>,
    {
        self.size.x * self.size.y
    }

    pub fn checked_total_area(&self) -> Option<C>
    where
        C: CheckedMul,
    {
        self.as_ref().checked_total_area_by_ref()
    }

    pub fn checked_move_point_unit(
        self,
        point: CoordPair<C>,
        direction: Direction,
    ) -> Result<CoordPair<C>, InvalidRectPoint<C>>
    where
        C: Add<Output = C> + Sub<Output = C> + One,
        C: Clone + PartialOrd + fmt::Display,
    {
        self.checked_move_point_by(point, One::one(), direction)
    }

    pub fn checked_move_point_by(
        self,
        point: CoordPair<C>,
        magnitude: C,
        direction: Direction,
    ) -> Result<CoordPair<C>, InvalidRectPoint<C>>
    where
        C: Add<Output = C> + Sub<Output = C>,
        C: Clone + PartialOrd + fmt::Display,
    {
        if self.clone().contains_point(point.clone()) {
            Ok(point.move_by(magnitude, direction))
        } else {
            Err(InvalidRectPoint { point, rect: self })
        }
    }
}

impl<'a, C> Rect<&'a C> {
    pub fn copied(self) -> Rect<C>
    where
        C: Copy,
    {
        Rect { top_left: self.top_left.copied(), size: self.size.copied() }
    }

    pub fn cloned(self) -> Rect<C>
    where
        C: Clone,
    {
        Rect { top_left: self.top_left.cloned(), size: self.size.cloned() }
    }

    pub fn checked_bottom_right_by_ref(self) -> Option<CoordPair<C>>
    where
        C: CheckedAdd,
    {
        self.top_left.checked_add_by_ref(self.size)
    }

    pub fn checked_total_area_by_ref(self) -> Option<C>
    where
        C: CheckedMul,
    {
        self.size.x.checked_mul(self.size.y)
    }
}

impl<'a, C> Rect<&'a mut C> {
    pub fn copied(self) -> Rect<C>
    where
        C: Copy,
    {
        Rect { top_left: self.top_left.copied(), size: self.size.copied() }
    }

    pub fn cloned(self) -> Rect<C>
    where
        C: Clone,
    {
        Rect { top_left: self.top_left.cloned(), size: self.size.cloned() }
    }

    pub fn share(self) -> Rect<&'a C> {
        Rect { top_left: self.top_left.share(), size: self.size.share() }
    }
}

impl<C> fmt::Display for Rect<C>
where
    C: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.top_left, self.size)
    }
}
