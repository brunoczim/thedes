use std::{
    fmt,
    ops::{Add, Div, Mul, Rem, Sub},
};

use num::{traits::CheckedRem, CheckedAdd, CheckedDiv, CheckedMul, One};
use thiserror::Error;

use crate::{axis::Direction, coords::CoordPair};

#[derive(Debug, Error)]
#[error("Invalid point {point} for rectangle {rect}")]
pub struct InvalidPoint<C>
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
        InvalidPoint<C>,
    ),
    #[error("Arithmetic overflow computing area for rectangle of size {size}")]
    Overflow { size: CoordPair<C> },
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
        self.top_left + self.size
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
            .all(|(start, size, coord)| coord >= start && coord - start < size)
    }

    pub fn checked_horz_area_down_to(
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
            Err(InvalidPoint { point, rect: self })?
        }
    }

    pub fn horz_area_down_to(self, point: CoordPair<C>) -> C
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
    ) -> Result<CoordPair<C>, InvalidPoint<C>>
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
    ) -> Result<CoordPair<C>, InvalidPoint<C>>
    where
        C: Add<Output = C> + Sub<Output = C>,
        C: Clone + PartialOrd + fmt::Display,
    {
        if self.clone().contains_point(point.clone()) {
            Ok(point.move_by(magnitude, direction))
        } else {
            Err(InvalidPoint { point, rect: self })
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
