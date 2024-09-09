use std::convert::Infallible;

use crate::{
    color::{self, ColorPair},
    grapheme,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tile {
    pub colors: ColorPair,
    pub grapheme: grapheme::Id,
}

pub trait Mutation {
    fn mutate_tile(self, input: Tile) -> Tile;
}

impl<F> Mutation for F
where
    F: FnOnce(Tile) -> Tile,
{
    fn mutate_tile(self, input: Tile) -> Tile {
        self(input)
    }
}

impl Mutation for Tile {
    fn mutate_tile(self, _input: Tile) -> Tile {
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

pub trait TryMutation {
    type Error;

    fn try_mutate_tile(self, input: Tile) -> Result<Tile, Self::Error>;
}

impl<F, E> TryMutation for F
where
    F: FnOnce(Tile) -> Result<Tile, E>,
{
    type Error = E;

    fn try_mutate_tile(self, input: Tile) -> Result<Tile, E> {
        self(input)
    }
}

impl TryMutation for Tile {
    type Error = Infallible;

    fn try_mutate_tile(self, _input: Tile) -> Result<Tile, Self::Error> {
        Ok(self)
    }
}

pub trait TryMutationExt: TryMutation {
    fn try_then<N>(self, after: N) -> Then<Self, N>
    where
        Self: Sized,
        N: TryMutation<Error = Self::Error>,
    {
        Then { before: self, after }
    }
}

impl<M> TryMutationExt for M where M: TryMutation + ?Sized {}

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
    fn mutate_tile(self, input: Tile) -> Tile {
        self.after.mutate_tile(self.before.mutate_tile(input))
    }
}

impl<M, N> TryMutation for Then<M, N>
where
    M: TryMutation,
    N: TryMutation<Error = M::Error>,
{
    type Error = M::Error;

    fn try_mutate_tile(self, input: Tile) -> Result<Tile, Self::Error> {
        self.after.try_mutate_tile(self.before.try_mutate_tile(input)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetGrapheme(pub grapheme::Id);

impl Mutation for SetGrapheme {
    fn mutate_tile(self, mut input: Tile) -> Tile {
        input.grapheme = self.0;
        input
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MutateColors<M>(pub M)
where
    M: color::Mutation;

impl<M> Mutation for MutateColors<M>
where
    M: color::Mutation,
{
    fn mutate_tile(self, mut input: Tile) -> Tile {
        input.colors = self.0.mutate_colors(input.colors);
        input
    }
}
