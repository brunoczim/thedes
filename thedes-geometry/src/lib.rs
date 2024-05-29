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
        Rem,
        RemAssign,
        Sub,
        SubAssign,
    },
};

use num::{One, Zero};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid point {point} for rectangle of size {rect_size}")]
pub struct InvalidRectPoint<T>
where
    T: fmt::Display,
{
    pub point: CoordPair<T>,
    pub rect_size: CoordPair<T>,
}

#[derive(Debug, Error)]
#[error("invalid line point ({line_point}) for rectangle of size {rect_size}")]
pub struct InvalidLinePoint<T>
where
    T: fmt::Display,
{
    pub line_point: T,
    pub rect_size: CoordPair<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    Y,
    X,
}

impl Axis {
    pub const ALL: [Self; 2] = [Self::Y, Self::X];
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

    pub fn as_rect_to_line(
        self,
        target_point: Self,
    ) -> Result<C, InvalidRectPoint<C>>
    where
        C: Add<Output = C> + Mul<Output = C> + Ord + fmt::Display,
    {
        if self
            .as_ref()
            .zip2(target_point.as_ref())
            .all(|(size, point)| size > point)
        {
            Ok(self.as_rect_to_line_unchecked(target_point))
        } else {
            Err(InvalidRectPoint { point: target_point, rect_size: self })
        }
    }

    pub fn as_rect_to_line_unchecked(self, target_point: Self) -> C
    where
        C: Add<Output = C> + Mul<Output = C>,
    {
        self.x * target_point.y + target_point.x
    }

    pub fn as_rect_from_line(
        self,
        line_point: C,
    ) -> Result<CoordPair<C>, InvalidLinePoint<C>>
    where
        C: Div<Output = C> + Rem<Output = C> + Mul<Output = C>,
        C: Ord + Clone + fmt::Display,
    {
        if line_point.clone() < self.x.clone() * self.y.clone() {
            Ok(self.as_rect_from_line_unchecked(line_point))
        } else {
            Err(InvalidLinePoint { rect_size: self, line_point })
        }
    }

    pub fn as_rect_from_line_unchecked(self, line_point: C) -> CoordPair<C>
    where
        C: Div<Output = C> + Rem<Output = C> + Clone,
    {
        CoordPair {
            y: line_point.clone() / self.x.clone(),
            x: line_point % self.x,
        }
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
