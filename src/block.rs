use crate::{
    entity,
    error::GameResult,
    orient::Camera,
    storage::save::SavedGame,
    terminal,
};
use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};
use std::{collections::HashSet, fmt::Write};

const EMPTY_WEIGHT: usize = 15;
const WALL_WEIGHT: usize = 1;

const WEIGHTS: &'static [(Kind, usize)] =
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
pub struct Dist {
    weighted: WeightedIndex<usize>,
}

impl Dist {
    pub fn new() -> Self {
        let weighted =
            WeightedIndex::new(WEIGHTS.iter().map(|&(_, weight)| weight))
                .expect("Could not create weighted index");
        Self { weighted }
    }
}

impl Distribution<Block> for Dist {
    fn sample<R>(&self, rng: &mut R) -> Block
    where
        R: Rng + ?Sized,
    {
        match WEIGHTS[rng.sample(&self.weighted)] {
            (Kind::Empty, _) => Block::Empty,
            (Kind::Wall, _) => Block::Wall,
        }
    }
}
