use std::convert::Infallible;

use thiserror::Error;

use crate::{
    color::{ColorPair, mutation::ColorMutationError},
    grapheme,
    mutation::{Mutable, Mutation},
};

#[derive(Debug, Error)]
pub enum TileMutationError {
    #[error("Failed to mutate color")]
    Color(
        #[source]
        #[from]
        ColorMutationError,
    ),
}

impl From<Infallible> for TileMutationError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tile {
    pub colors: ColorPair,
    pub grapheme: grapheme::Id,
}

impl Default for Tile {
    fn default() -> Self {
        Self { colors: ColorPair::default(), grapheme: grapheme::Id::from(' ') }
    }
}

impl Mutable for Tile {
    type Error = TileMutationError;
}

impl Mutable for grapheme::Id {
    type Error = Infallible;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MutateColors<M>(pub M);

impl<M> Mutation<Tile> for MutateColors<M>
where
    M: Mutation<ColorPair>,
{
    fn mutate(
        self,
        mut target: Tile,
    ) -> Result<Tile, <Tile as Mutable>::Error> {
        let Self(mutation) = self;
        target.colors = mutation.mutate(target.colors)?;
        Ok(target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MutateGrapheme<M>(pub M);

impl<M> Mutation<Tile> for MutateGrapheme<M>
where
    M: Mutation<grapheme::Id>,
{
    fn mutate(
        self,
        mut target: Tile,
    ) -> Result<Tile, <Tile as Mutable>::Error> {
        let Self(mutation) = self;
        target.grapheme = mutation.mutate(target.grapheme)?;
        Ok(target)
    }
}
