use thiserror::Error;

use crate::mutation::{Mutable, Mutation};

use super::{
    ApproxBrightness,
    Brightness,
    BrightnessError,
    Color,
    ColorPair,
    MutableApproxBrightness,
};

pub type BrightnessMutationError = BrightnessError;

#[derive(Debug, Error)]
pub enum ColorMutationError {
    #[error("Failed to mutate color brightness")]
    Brightness(
        #[from]
        #[source]
        BrightnessError,
    ),
}

impl Mutable for Brightness {
    type Error = BrightnessMutationError;
}

impl Mutable for Color {
    type Error = ColorMutationError;
}

impl Mutable for ColorPair {
    type Error = ColorMutationError;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AdaptTo(pub Brightness);

impl Mutation<Brightness> for AdaptTo {
    fn mutate(
        self,
        target: Brightness,
    ) -> Result<Brightness, <Brightness as Mutable>::Error> {
        let Self(reference_brightness) = self;
        Ok(target.adapt_to(reference_brightness))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ContrastTo(pub Brightness);

impl Mutation<Brightness> for ContrastTo {
    fn mutate(
        self,
        target: Brightness,
    ) -> Result<Brightness, <Brightness as Mutable>::Error> {
        let Self(reference_brightness) = self;
        Ok(target.contrast_to(reference_brightness))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MutateBrightness<M>(pub M);

impl<M> Mutation<Color> for MutateBrightness<M>
where
    M: Mutation<Brightness>,
{
    fn mutate(
        self,
        mut target: Color,
    ) -> Result<Color, <Color as Mutable>::Error> {
        let Self(mutation) = self;
        let brightness = target.approx_brightness()?;
        target.set_approx_brightness(mutation.mutate(brightness)?)?;
        Ok(target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MutateBg<M>(pub M);

impl<M> Mutation<ColorPair> for MutateBg<M>
where
    M: Mutation<Color>,
{
    fn mutate(
        self,
        mut target: ColorPair,
    ) -> Result<ColorPair, <ColorPair as Mutable>::Error> {
        let Self(mutation) = self;
        target.background = mutation.mutate(target.background)?;
        Ok(target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MutateFg<M>(pub M);

impl<M> Mutation<ColorPair> for MutateFg<M>
where
    M: Mutation<Color>,
{
    fn mutate(
        self,
        mut target: ColorPair,
    ) -> Result<ColorPair, <ColorPair as Mutable>::Error> {
        let Self(mutation) = self;
        target.foreground = mutation.mutate(target.foreground)?;
        Ok(target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AdaptFgToBg;

impl Mutation<ColorPair> for AdaptFgToBg {
    fn mutate(
        self,
        mut target: ColorPair,
    ) -> Result<ColorPair, <ColorPair as Mutable>::Error> {
        target.foreground.set_adapt_to(target.background)?;
        Ok(target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AdaptBgToFg;

impl Mutation<ColorPair> for AdaptBgToFg {
    fn mutate(
        self,
        mut target: ColorPair,
    ) -> Result<ColorPair, <ColorPair as Mutable>::Error> {
        target.background.set_adapt_to(target.foreground)?;
        Ok(target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ContrastFgToBg;

impl Mutation<ColorPair> for ContrastFgToBg {
    fn mutate(
        self,
        mut target: ColorPair,
    ) -> Result<ColorPair, <ColorPair as Mutable>::Error> {
        target.foreground.set_contrast_to(target.background)?;
        Ok(target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ContrastBgToFg;

impl Mutation<ColorPair> for ContrastBgToFg {
    fn mutate(
        self,
        mut target: ColorPair,
    ) -> Result<ColorPair, <ColorPair as Mutable>::Error> {
        target.background.set_contrast_to(target.foreground)?;
        Ok(target)
    }
}
