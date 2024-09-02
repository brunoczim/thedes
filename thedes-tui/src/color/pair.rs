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
    fn mutate_colors(self, pair: ColorPair) -> ColorPair;
}

impl Mutation for ColorPair {
    fn mutate_colors(self, _pair: ColorPair) -> ColorPair {
        self
    }
}

pub trait MutationExt: Mutation {
    fn then<N>(self, after: N) -> Then<Self, N>
    where
        Self: Sized,
        N: Mutation,
    {
        Then { before: self, after }
    }
}

impl<M> MutationExt for M where M: Mutation + ?Sized {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Then<M, N> {
    before: M,
    after: N,
}

impl<M, N> Mutation for Then<M, N>
where
    M: Mutation,
    N: Mutation,
{
    fn mutate_colors(self, input: ColorPair) -> ColorPair {
        self.after.mutate_colors(self.before.mutate_colors(input))
    }
}

/// Updates the foreground of a pair of colors ([`Color2`]) to the given color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetFg(pub Color);

impl Mutation for SetFg {
    fn mutate_colors(self, pair: ColorPair) -> ColorPair {
        ColorPair { foreground: self.0, background: pair.background }
    }
}

/// Updates the background of a pair of colors ([`Color2`]) to the given color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetBg(pub Color);

impl Mutation for SetBg {
    fn mutate_colors(self, pair: ColorPair) -> ColorPair {
        ColorPair { foreground: pair.foreground, background: self.0 }
    }
}

/// Adapts the brightness of the foreground color to match the background color
/// of a pair of colors ([`Color2`]). This means foreground is modified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdaptFgToBg;

impl Mutation for AdaptFgToBg {
    fn mutate_colors(self, pair: ColorPair) -> ColorPair {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdaptBgToFg;

impl Mutation for AdaptBgToFg {
    fn mutate_colors(self, pair: ColorPair) -> ColorPair {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContrastFgWithBg;

impl Mutation for ContrastFgWithBg {
    fn mutate_colors(self, pair: ColorPair) -> ColorPair {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContrastBgWithFg;

impl Mutation for ContrastBgWithFg {
    fn mutate_colors(self, pair: ColorPair) -> ColorPair {
        ColorPair {
            foreground: pair.foreground,
            background: pair
                .background
                .with_approx_brightness(!pair.foreground.approx_brightness()),
        }
    }
}
