use std::{rc::Rc, sync::Arc};

use thedes_domain::matter::Ground;
use thedes_tui::color::{Color, EightBitColor, LegacyRgb};

pub trait Background {
    fn base_color(&self) -> Color;
}

impl<'a, T> Background for &'a T
where
    T: Background + ?Sized,
{
    fn base_color(&self) -> Color {
        (**self).base_color()
    }
}

impl<'a, T> Background for &'a mut T
where
    T: Background + ?Sized,
{
    fn base_color(&self) -> Color {
        (**self).base_color()
    }
}

impl<T> Background for Box<T>
where
    T: Background + ?Sized,
{
    fn base_color(&self) -> Color {
        (**self).base_color()
    }
}

impl<T> Background for Rc<T>
where
    T: Background + ?Sized,
{
    fn base_color(&self) -> Color {
        (**self).base_color()
    }
}

impl<T> Background for Arc<T>
where
    T: Background + ?Sized,
{
    fn base_color(&self) -> Color {
        (**self).base_color()
    }
}

impl Background for Ground {
    fn base_color(&self) -> Color {
        let cmy_color = match self {
            Self::Sand => LegacyRgb::new(4, 4, 1),
            Self::Grass => LegacyRgb::new(1, 5, 2),
            Self::Stone => LegacyRgb::new(2, 2, 2),
        };

        EightBitColor::from(cmy_color).into()
    }
}
