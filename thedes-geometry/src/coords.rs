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
    traits::{CheckedRem, SaturatingAdd, SaturatingMul, SaturatingSub},
    CheckedAdd,
    CheckedDiv,
    CheckedMul,
    CheckedSub,
    Integer,
    One,
    Zero,
};

use crate::{
    axis::{Axis, Direction},
    rect::Rect,
};

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

    pub fn checked_add_to(&self, other: &C) -> Option<Self>
    where
        C: CheckedAdd,
    {
        self.as_ref().checked_add_by_ref_to(other)
    }

    pub fn checked_sub_except(&self, other: &C) -> Option<Self>
    where
        C: CheckedSub,
    {
        self.as_ref().checked_sub_by_ref_except(other)
    }

    pub fn checked_sub_from(&self, other: &C) -> Option<Self>
    where
        C: CheckedSub,
    {
        self.as_ref().checked_sub_by_ref_from(other)
    }

    pub fn checked_mul_scalar(&self, other: &C) -> Option<Self>
    where
        C: CheckedMul,
    {
        self.as_ref().checked_mul_by_ref_scalar(other)
    }

    pub fn checked_div_by(&self, other: &C) -> Option<Self>
    where
        C: CheckedDiv,
    {
        self.as_ref().checked_div_by_ref_by(other)
    }

    pub fn checked_div_on(&self, other: &C) -> Option<Self>
    where
        C: CheckedDiv,
    {
        self.as_ref().checked_div_by_ref_on(other)
    }

    pub fn checked_rem_by(&self, other: &C) -> Option<Self>
    where
        C: CheckedRem,
    {
        self.as_ref().checked_rem_by_ref_by(other)
    }

    pub fn checked_rem_on(&self, other: &C) -> Option<Self>
    where
        C: CheckedRem,
    {
        self.as_ref().checked_rem_by_ref_on(other)
    }

    pub fn saturating_add_to(&self, other: &C) -> Self
    where
        C: SaturatingAdd,
    {
        self.as_ref().saturating_add_by_ref_to(other)
    }

    pub fn saturating_sub_except(&self, other: &C) -> Self
    where
        C: SaturatingSub,
    {
        self.as_ref().saturating_sub_by_ref_except(other)
    }

    pub fn saturating_sub_from(&self, other: &C) -> Self
    where
        C: SaturatingSub,
    {
        self.as_ref().saturating_sub_by_ref_from(other)
    }

    pub fn saturating_mul_scalar(&self, other: &C) -> Self
    where
        C: SaturatingMul,
    {
        self.as_ref().saturating_mul_by_ref_scalar(other)
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
        direction.move_unit(self)
    }

    pub fn checked_move_unit(&self, direction: Direction) -> Option<Self>
    where
        C: CheckedAdd + CheckedSub + One + Clone,
    {
        direction.checked_move_unit(self)
    }

    pub fn saturating_move_unit(&self, direction: Direction) -> Self
    where
        C: SaturatingAdd + SaturatingSub + One + Clone,
    {
        direction.saturating_move_unit(self)
    }

    pub fn move_by(self, magnitude: C, direction: Direction) -> Self
    where
        C: Add<Output = C> + Sub<Output = C>,
    {
        direction.move_by(magnitude, self)
    }

    pub fn checked_move_by(
        &self,
        magnitude: &C,
        direction: Direction,
    ) -> Option<Self>
    where
        C: CheckedAdd + CheckedSub + Clone,
    {
        direction.checked_move_by(magnitude, self)
    }

    pub fn saturating_move_by(
        &self,
        magnitude: &C,
        direction: Direction,
    ) -> Self
    where
        C: SaturatingAdd + SaturatingSub + Clone,
    {
        direction.saturating_move_by(magnitude, self)
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

    pub fn checked_add_by_ref_to(self, other: &C) -> Option<CoordPair<C>>
    where
        C: CheckedAdd,
    {
        self.checked_add_by_ref(CoordPair::from_axes(|_| other))
    }

    pub fn checked_sub_by_ref(self, other: Self) -> Option<CoordPair<C>>
    where
        C: CheckedSub,
    {
        self.zip2_with(other, C::checked_sub).transpose()
    }

    pub fn checked_sub_by_ref_except(self, other: &C) -> Option<CoordPair<C>>
    where
        C: CheckedSub,
    {
        self.checked_sub_by_ref(CoordPair::from_axes(|_| other))
    }

    pub fn checked_sub_by_ref_from(self, other: &C) -> Option<CoordPair<C>>
    where
        C: CheckedSub,
    {
        CoordPair::from_axes(|_| other).checked_sub_by_ref(self)
    }

    pub fn checked_mul_by_ref(self, other: Self) -> Option<CoordPair<C>>
    where
        C: CheckedMul,
    {
        self.zip2_with(other, C::checked_mul).transpose()
    }

    pub fn checked_mul_by_ref_scalar(self, other: &C) -> Option<CoordPair<C>>
    where
        C: CheckedMul,
    {
        self.checked_mul_by_ref(CoordPair::from_axes(|_| other))
    }

    pub fn checked_div_by_ref(self, other: Self) -> Option<CoordPair<C>>
    where
        C: CheckedDiv,
    {
        self.zip2_with(other, C::checked_div).transpose()
    }

    pub fn checked_div_by_ref_by(self, other: &C) -> Option<CoordPair<C>>
    where
        C: CheckedDiv,
    {
        self.checked_div_by_ref(CoordPair::from_axes(|_| other))
    }

    pub fn checked_div_by_ref_on(self, other: &C) -> Option<CoordPair<C>>
    where
        C: CheckedDiv,
    {
        CoordPair::from_axes(|_| other).checked_div_by_ref(self)
    }

    pub fn checked_rem_by_ref(self, other: Self) -> Option<CoordPair<C>>
    where
        C: CheckedRem,
    {
        self.zip2_with(other, C::checked_rem).transpose()
    }

    pub fn checked_rem_by_ref_by(self, other: &C) -> Option<CoordPair<C>>
    where
        C: CheckedRem,
    {
        self.checked_rem_by_ref(CoordPair::from_axes(|_| other))
    }

    pub fn checked_rem_by_ref_on(self, other: &C) -> Option<CoordPair<C>>
    where
        C: CheckedRem,
    {
        CoordPair::from_axes(|_| other).checked_rem_by_ref(self)
    }

    pub fn saturating_add_by_ref(self, other: Self) -> CoordPair<C>
    where
        C: SaturatingAdd,
    {
        self.zip2_with(other, C::saturating_add)
    }

    pub fn saturating_add_by_ref_to(self, other: &C) -> CoordPair<C>
    where
        C: SaturatingAdd,
    {
        self.saturating_add_by_ref(CoordPair::from_axes(|_| other))
    }

    pub fn saturating_sub_by_ref(self, other: Self) -> CoordPair<C>
    where
        C: SaturatingSub,
    {
        self.zip2_with(other, C::saturating_sub)
    }

    pub fn saturating_sub_by_ref_except(self, other: &C) -> CoordPair<C>
    where
        C: SaturatingSub,
    {
        self.saturating_sub_by_ref(CoordPair::from_axes(|_| other))
    }

    pub fn saturating_sub_by_ref_from(self, other: &C) -> CoordPair<C>
    where
        C: SaturatingSub,
    {
        self.saturating_sub_by_ref(CoordPair::from_axes(|_| other))
    }

    pub fn saturating_mul_by_ref(self, other: Self) -> CoordPair<C>
    where
        C: SaturatingMul,
    {
        self.zip2_with(other, C::saturating_mul)
    }

    pub fn saturating_mul_by_ref_scalar(self, other: &C) -> CoordPair<C>
    where
        C: SaturatingMul,
    {
        self.saturating_mul_by_ref(CoordPair::from_axes(|_| other))
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
        direction.checked_move_unit_ref_by(self)
    }

    pub fn saturating_move_unit_by_ref(
        self,
        direction: Direction,
    ) -> CoordPair<C>
    where
        C: SaturatingAdd + SaturatingSub + One + Clone,
    {
        direction.saturating_move_unit_by_ref(self)
    }

    pub fn checked_move_by_ref_by(
        self,
        magnitude: &C,
        direction: Direction,
    ) -> Option<CoordPair<C>>
    where
        C: CheckedAdd + CheckedSub + Clone,
    {
        direction.checked_move_by_ref_by(magnitude, self)
    }

    pub fn saturating_move_by_ref_by(
        self,
        magnitude: &C,
        direction: Direction,
    ) -> CoordPair<C>
    where
        C: SaturatingAdd + SaturatingSub + Clone,
    {
        direction.saturating_move_by_ref_by(magnitude, self)
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

impl<C> SaturatingAdd for CoordPair<C>
where
    C: SaturatingAdd,
{
    fn saturating_add(&self, other: &Self) -> Self {
        self.as_ref().saturating_add_by_ref(other.as_ref())
    }
}

impl<C> SaturatingSub for CoordPair<C>
where
    C: SaturatingSub,
{
    fn saturating_sub(&self, other: &Self) -> Self {
        self.as_ref().saturating_sub_by_ref(other.as_ref())
    }
}

impl<C> SaturatingMul for CoordPair<C>
where
    C: SaturatingMul,
{
    fn saturating_mul(&self, other: &Self) -> Self {
        self.as_ref().saturating_mul_by_ref(other.as_ref())
    }
}
