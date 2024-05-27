use crate::{color::ColorPair, grapheme};

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

pub trait TryMutation {
    type Error;

    fn try_mutate_tile(self, input: Tile) -> Result<Tile, Self::Error>;
}

impl<F, E> TryMutation for F
where
    F: FnOnce(Tile) -> Result<Tile, E>,
{
    type Error = E;

    fn try_mutate_tile(self, input: Tile) -> Result<Tile, Self::Error> {
        self(input)
    }
}
