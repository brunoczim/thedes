use crate::color::{ApproxBrightness, BasicColor, Color};
use std::ops::Not;

/// A pair of colors (foreground and background).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorPair {
    /// The foreground of this pair.
    pub foreground: Color,
    /// The background of this pair.
    pub background: Color,
}

impl ColorPair {
    /// Just a convenience method for creating color pairs with conversion.
    pub fn new<F, B>(foreground: F, background: B) -> Self
    where
        F: Into<Color>,
        B: Into<Color>,
    {
        Self { foreground: foreground.into(), background: background.into() }
    }
}

impl Default for ColorPair {
    fn default() -> Self {
        Self::new(BasicColor::White, BasicColor::Black)
    }
}

impl Not for ColorPair {
    type Output = ColorPair;

    fn not(self) -> Self::Output {
        ColorPair { foreground: !self.foreground, background: !self.background }
    }
}

/// A function that updates a [`Color2`].
pub trait Mutation {
    /// Receives a pair of color and yields a new one.
    fn update(&self, pair: ColorPair) -> ColorPair;
}

impl Mutation for ColorPair {
    fn update(&self, _pair: ColorPair) -> ColorPair {
        *self
    }
}

impl<'this, T> Mutation for &'this T
where
    T: Mutation,
{
    fn update(&self, pair: ColorPair) -> ColorPair {
        (**self).update(pair)
    }
}

/// Updates the foreground of a pair of colors ([`Color2`]) to the given color.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpdateFg(pub Color);

impl Mutation for UpdateFg {
    fn update(&self, pair: ColorPair) -> ColorPair {
        ColorPair { foreground: self.0, background: pair.background }
    }
}

/// Updates the background of a pair of colors ([`Color2`]) to the given color.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpdateBg(pub Color);

impl Mutation for UpdateBg {
    fn update(&self, pair: ColorPair) -> ColorPair {
        ColorPair { foreground: pair.foreground, background: self.0 }
    }
}

/// Adapts the brightness of the foreground color to match the background color
/// of a pair of colors ([`Color2`]). This means foreground is modified.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdaptFgToBg;

impl Mutation for AdaptFgToBg {
    fn update(&self, pair: ColorPair) -> ColorPair {
        ColorPair {
            background: pair.background,
            foreground: pair
                .foreground
                .with_approx_brightness(pair.background.approx_brightness()),
        }
    }
}

/// Adapts the brightness of the background color to match the foreground color
/// of a pair of colors ([`Color2`]). This means background is modified.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdaptBgToFg;

impl Mutation for AdaptBgToFg {
    fn update(&self, pair: ColorPair) -> ColorPair {
        ColorPair {
            foreground: pair.foreground,
            background: pair
                .background
                .with_approx_brightness(pair.foreground.approx_brightness()),
        }
    }
}

/// Contrasts the brightness of the foreground color against the background
/// color of a pair of colors ([`Color2`]). This means foreground is modified.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContrastFgWithBg;

impl Mutation for ContrastFgWithBg {
    fn update(&self, pair: ColorPair) -> ColorPair {
        ColorPair {
            background: pair.background,
            foreground: pair
                .foreground
                .with_approx_brightness(!pair.background.approx_brightness()),
        }
    }
}

/// Contrasts the brightness of the background color against the foreground
/// color of a pair of colors ([`Color2`]). This means background is modified.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContrastBgWithFg;

impl Mutation for ContrastBgWithFg {
    fn update(&self, pair: ColorPair) -> ColorPair {
        ColorPair {
            foreground: pair.foreground,
            background: pair
                .background
                .with_approx_brightness(!pair.foreground.approx_brightness()),
        }
    }
}

macro_rules! impl_tuple {
    {} => {};
    { $name:ident $(, $names:ident)* } => {
        impl<$name $(, $names)*> Mutation for ($name, $($names),*)
        where
            $name: Mutation,
            $($names: Mutation),*
        {
            fn update(&self, pair: ColorPair) -> ColorPair {
                #[allow(non_snake_case)]
                let ($name, $($names),*) = self;
                let result = $name.update(pair);
                $(let result = $names.update(result);)*
                result
            }
        }

        impl_tuple! { $($names),* }
    };
}

impl_tuple! {
    A0, A1, A2, A3, A4, A5, A6, A7,
    B0, B1, B2, B3, B4, B5, B6, B7,
    C0, C1, C2, C3, C4, C5, C6, C7,
    D0, D1, D2, D3, D4, D5, D6, D7
}
