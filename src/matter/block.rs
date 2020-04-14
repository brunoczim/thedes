use crate::{
    coord::{Camera, Coord2, Nat},
    entity,
    error::Result,
    graphics::{Color, Foreground, Grapheme},
    rand::{NoiseGen, NoiseInput, NoiseProcessor, Seed, WeightedNoise},
    storage::save::{self, SavedGame},
    terminal,
};
use rand::rngs::StdRng;
use std::collections::HashSet;
use tokio::task;

const SEED_SALT: u128 = 0xE6ADC41EE9FFBA2BCD59A18919921DFE;

const WEIGHTS: &'static [(Block, u64)] = &[(Block::Empty, 3), (Block::Wall, 2)];

/// Kind of a block.
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
pub enum Block {
    /// Empty.
    Empty,
    /// Wall block.
    Wall,
    /// An entity's physical part.
    Entity(entity::Physical),
}

impl Block {
    /// Renders this block on the screen.
    pub async fn render<'guard>(
        &self,
        pos: Coord2<Nat>,
        camera: Camera,
        screen: &mut terminal::Screen<'guard>,
        game: &SavedGame,
        rendered_entities: &mut HashSet<entity::Physical>,
    ) -> Result<()> {
        if let Some(pos) = camera.convert(pos) {
            let bg = screen.get(pos).colors.bg;
            let grapheme = match self {
                Block::Empty => Grapheme::new_lossy(" "),
                Block::Wall => Grapheme::new_lossy("+"),
                Block::Entity(physical) => {
                    if rendered_entities.insert(physical.clone()) {
                        physical.render(camera, screen, game).await?;
                    }
                    return Ok(());
                },
            };
            let fg = Foreground { grapheme, color: Color::White };
            screen.set(pos, fg.make_tile(bg));
        }

        Ok(())
    }
}

/// A type that computes blocks from noise.
#[derive(Debug, Clone)]
pub struct FromNoise {
    weighted: WeightedNoise,
}

impl FromNoise {
    /// Initializes this processor.
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

    fn process(&self, input: I, gen: &NoiseGen) -> Self::Output {
        let index = self.weighted.process(input, gen);
        let (block, _) = &WEIGHTS[index];
        block.clone()
    }
}

/// A persitent map of blocks.
#[derive(Debug, Clone)]
pub struct Map {
    tree: sled::Tree,
    noise_gen: NoiseGen,
    noise_proc: FromNoise,
}

impl Map {
    /// Creates a new map given a tree that stores blocks using coordinate pairs
    /// as keys. A seed is provided to create the noise function.
    pub fn new(db: &sled::Db, seed: Seed) -> Result<Self> {
        let tree = task::block_in_place(|| db.open_tree("block::Map"))?;
        Ok(Self {
            tree,
            noise_gen: {
                let mut noise = seed.make_noise_gen::<_, StdRng>(SEED_SALT);
                noise.sensitivity = 0.005;
                noise
            },
            noise_proc: FromNoise::new(),
        })
    }

    /// Sets a block at a given point.
    pub async fn set(&self, point: Coord2<Nat>, value: &Block) -> Result<()> {
        let point_vec = save::encode(point)?;
        let block_vec = save::encode(value)?;
        task::block_in_place(|| self.tree.insert(point_vec, block_vec))?;
        Ok(())
    }

    /// Gets a block at a given point.
    pub async fn get(&self, point: Coord2<Nat>) -> Result<Block> {
        let point_vec = save::encode(point)?;
        let res = task::block_in_place(|| self.tree.get(point_vec));

        match res? {
            Some(bytes) => Ok(save::decode(&bytes)?),
            None => {
                let block = self.noise_proc.process(point, &self.noise_gen);
                self.set(point, &block).await?;
                Ok(block)
            },
        }
    }
}
