pub mod game;

use std::{rc::Rc, sync::Arc};

use thedes_domain::geometry::{CoordPair, Rect};

use crate::tile;

pub trait Viewable {
    type Error: std::error::Error + Send + Sync + 'static;

    fn render<R>(&self, rect: Rect, renderer: R) -> Result<(), Self::Error>
    where
        R: Renderer;
}

impl<'a, T> Viewable for &'a T
where
    T: Viewable + ?Sized,
{
    type Error = T::Error;

    fn render<R>(&self, rect: Rect, renderer: R) -> Result<(), Self::Error>
    where
        R: Renderer,
    {
        (**self).render(rect, renderer)
    }
}

impl<'a, T> Viewable for &'a mut T
where
    T: Viewable + ?Sized,
{
    type Error = T::Error;

    fn render<R>(&self, rect: Rect, renderer: R) -> Result<(), Self::Error>
    where
        R: Renderer,
    {
        (**self).render(rect, renderer)
    }
}

impl<T> Viewable for Box<T>
where
    T: Viewable + ?Sized,
{
    type Error = T::Error;

    fn render<R>(&self, rect: Rect, renderer: R) -> Result<(), Self::Error>
    where
        R: Renderer,
    {
        (**self).render(rect, renderer)
    }
}

impl<T> Viewable for Rc<T>
where
    T: Viewable + ?Sized,
{
    type Error = T::Error;

    fn render<R>(&self, rect: Rect, renderer: R) -> Result<(), Self::Error>
    where
        R: Renderer,
    {
        (**self).render(rect, renderer)
    }
}

impl<T> Viewable for Arc<T>
where
    T: Viewable + ?Sized,
{
    type Error = T::Error;

    fn render<R>(&self, rect: Rect, renderer: R) -> Result<(), Self::Error>
    where
        R: Renderer,
    {
        (**self).render(rect, renderer)
    }
}

pub trait Renderer {
    type TileRenderer<'r>: tile::Renderer
    where
        Self: 'r;

    fn tile_renderer<'r>(
        &'r mut self,
        position: CoordPair,
    ) -> Self::TileRenderer<'r>;
}

impl<'a, R> Renderer for &'a mut R
where
    R: Renderer + ?Sized,
{
    type TileRenderer<'r>
        = R::TileRenderer<'r>
    where
        &'a mut R: 'r;

    fn tile_renderer<'r>(
        &'r mut self,
        position: CoordPair,
    ) -> Self::TileRenderer<'r> {
        (**self).tile_renderer(position)
    }
}

impl<R> Renderer for Box<R>
where
    R: Renderer + ?Sized,
{
    type TileRenderer<'r>
        = R::TileRenderer<'r>
    where
        Box<R>: 'r;

    fn tile_renderer<'r>(
        &'r mut self,
        position: CoordPair,
    ) -> Self::TileRenderer<'r> {
        (**self).tile_renderer(position)
    }
}
