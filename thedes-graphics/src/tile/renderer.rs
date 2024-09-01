use super::{Background, Foreground};

pub trait Renderer {
    type Error: std::error::Error + Send + Sync + 'static;

    fn render_foreground<F>(
        &mut self,
        foreground: F,
    ) -> Result<(), Self::Error>
    where
        F: Foreground;

    fn render_background<B>(
        &mut self,
        background: B,
    ) -> Result<(), Self::Error>
    where
        B: Background;
}

impl<'a, R> Renderer for &'a mut R
where
    R: Renderer + ?Sized,
{
    type Error = R::Error;

    fn render_foreground<F>(&mut self, foreground: F) -> Result<(), Self::Error>
    where
        F: Foreground,
    {
        (**self).render_foreground(foreground)
    }

    fn render_background<B>(&mut self, background: B) -> Result<(), Self::Error>
    where
        B: Background,
    {
        (**self).render_background(background)
    }
}

impl<'a, R> Renderer for Box<R>
where
    R: Renderer + ?Sized,
{
    type Error = R::Error;

    fn render_foreground<F>(&mut self, foreground: F) -> Result<(), Self::Error>
    where
        F: Foreground,
    {
        (**self).render_foreground(foreground)
    }

    fn render_background<B>(&mut self, background: B) -> Result<(), Self::Error>
    where
        B: Background,
    {
        (**self).render_background(background)
    }
}
