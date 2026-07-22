use std::{rc::Rc, sync::Arc};

use thedes_geometry::orientation::Direction;
use thedes_tui::core::{
    color::{Color, LegacyRgb},
    grapheme::{self, NotGrapheme},
};

pub trait Foreground {
    fn base_color(&self) -> Color;

    fn grapheme(
        &self,
        graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme>;
}

impl<'a, T> Foreground for &'a T
where
    T: Foreground + ?Sized,
{
    fn base_color(&self) -> Color {
        (**self).base_color()
    }

    fn grapheme(
        &self,
        graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme> {
        (**self).grapheme(graphemes)
    }
}

impl<'a, T> Foreground for &'a mut T
where
    T: Foreground + ?Sized,
{
    fn base_color(&self) -> Color {
        (**self).base_color()
    }

    fn grapheme(
        &self,
        graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme> {
        (**self).grapheme(graphemes)
    }
}

impl<T> Foreground for Box<T>
where
    T: Foreground + ?Sized,
{
    fn base_color(&self) -> Color {
        (**self).base_color()
    }

    fn grapheme(
        &self,
        graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme> {
        (**self).grapheme(graphemes)
    }
}

impl<T> Foreground for Rc<T>
where
    T: Foreground + ?Sized,
{
    fn base_color(&self) -> Color {
        (**self).base_color()
    }

    fn grapheme(
        &self,
        graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme> {
        (**self).grapheme(graphemes)
    }
}

impl<T> Foreground for Arc<T>
where
    T: Foreground + ?Sized,
{
    fn base_color(&self) -> Color {
        (**self).base_color()
    }

    fn grapheme(
        &self,
        graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme> {
        (**self).grapheme(graphemes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PlayerHead;

impl Foreground for PlayerHead {
    fn base_color(&self) -> Color {
        LegacyRgb::new_saturating(0, 0, 0).into()
    }

    fn grapheme(
        &self,
        _graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme> {
        Ok(grapheme::Id::from('o'))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerPointer {
    pub facing: Direction,
}

impl Foreground for PlayerPointer {
    fn base_color(&self) -> Color {
        LegacyRgb::new_saturating(0, 0, 0).into()
    }

    fn grapheme(
        &self,
        _graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme> {
        let grapheme = match self.facing {
            Direction::Up => 'Λ',
            Direction::Left => '<',
            Direction::Down => 'V',
            Direction::Right => '>',
        };
        Ok(grapheme::Id::from(grapheme))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonsterHead {
    pub facing: Direction,
}

impl Foreground for MonsterHead {
    fn base_color(&self) -> Color {
        LegacyRgb::new_saturating(0, 0, 0).into()
    }

    fn grapheme(
        &self,
        _graphemes: &mut grapheme::Registry,
    ) -> Result<grapheme::Id, NotGrapheme> {
        let grapheme = match self.facing {
            Direction::Up => 'ɷ',
            Direction::Down => 'ო',
            Direction::Left => 'ɞ',
            Direction::Right => 'ʚ',
        };
        Ok(grapheme::Id::from(grapheme))
    }
}
