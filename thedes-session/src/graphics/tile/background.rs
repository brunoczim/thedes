use std::{rc::Rc, sync::Arc};

use thedes_domain::matter::Ground;
use thedes_tui::core::color::{Color, Rgb};

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
        let rgb_color = match self {
            Self::Grass => Rgb::new(0x00, 0xff, 0x80),
            Self::Sand => Rgb::new(0xff, 0xff, 0x80),
            Self::Stone => Rgb::new(0xc0, 0xc0, 0xc0),
        };

        rgb_color.into()
    }
}
