use crate::{
    entity,
    error::GameResult,
    orient::Camera,
    rand::{NoiseFnExt, NoiseInput, NoiseProcessor, WeightedNoise},
    storage::save::SavedGame,
    terminal,
};
use std::{collections::HashSet, fmt::Write};

const EMPTY_WEIGHT: u64 = 10;
const WALL_WEIGHT: u64 = 1;

const WEIGHTS: &'static [(Kind, u64)] =
    &[(Kind::Empty, EMPTY_WEIGHT), (Kind::Wall, WALL_WEIGHT)];

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
/// Kind of a block.
pub enum Kind {
    /// Empty.
    Empty,
    /// Wall block.
    Wall,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// A single block in the game.
pub enum Block {
    /// Empty.
    Empty,
    /// Wall block.
    Wall,
    /// Entity is occupying this block.
    Entity(entity::Id),
}

impl Block {
    /// Renders this block. If this is an entity piece, and has already been
    /// rendered, it is skipped.
    pub async fn render(
        &self,
        camera: Camera,
        term: &mut terminal::Handle,
        game: &SavedGame,
        rendered_entities: &mut HashSet<entity::Id>,
    ) -> GameResult<()> {
        match self {
            Self::Empty => write!(term, " ")?,
            Self::Wall => write!(term, "█")?,
            Self::Entity(id) => {
                if rendered_entities.insert(*id) {
                    game.entity(*id).await?.render(camera, term).await?
                }
            },
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FromNoise {
    weighted: WeightedNoise,
}

impl FromNoise {
    pub fn new() -> Self {
        let weighted =
            WeightedNoise::new(WEIGHTS.iter().map(|&(_, weight)| weight));
        Self { weighted }
    }
}

impl<I> NoiseProcessor<I> for FromNoise
where
    I: NoiseInput,
{
    type Output = Block;

    fn process<N>(&self, input: I, gen: &N) -> Self::Output
    where
        N: NoiseFnExt + ?Sized,
    {
        let index = self.weighted.process(input, gen);
        let (kind, _) = &WEIGHTS[index];

        match *kind {
            Kind::Empty => Block::Empty,
            Kind::Wall => Block::Wall,
        }
    }
}
