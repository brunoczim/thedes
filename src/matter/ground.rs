use crate::{
    coord::{Camera, Coord2, Nat},
    error::Result,
    graphics::Color,
    rand::{NoiseGen, NoiseInput, NoiseProcessor, Seed, WeightedNoise},
    storage::save,
    terminal,
};
use rand::rngs::StdRng;
use tokio::task;

const SEED_SALT: u128 = 0x7212E5AD960D877A02332BE4F063DF4D;

const LOW_WEIGHT: u64 = 1;
const MID_WEIGHT: u64 = 2;
const HIGH_WEIGHT: u64 = 3;

const WEIGHTS: &'static [(Ground, u64)] = &[
    (Ground::Rock, LOW_WEIGHT),
    (Ground::Grass, MID_WEIGHT),
    (Ground::Sand, LOW_WEIGHT),
    (Ground::Grass, LOW_WEIGHT),
    (Ground::Rock, HIGH_WEIGHT),
    (Ground::Sand, MID_WEIGHT),
    (Ground::Rock, LOW_WEIGHT),
    (Ground::Grass, HIGH_WEIGHT),
    (Ground::Rock, MID_WEIGHT),
    (Ground::Sand, LOW_WEIGHT),
    (Ground::Grass, MID_WEIGHT),
    (Ground::Sand, HIGH_WEIGHT),
    (Ground::Rock, LOW_WEIGHT),
    (Ground::Sand, LOW_WEIGHT),
    (Ground::Grass, LOW_WEIGHT),
];

/// A ground block (background color).
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
pub enum Ground {
    /// This block's ground is grass.
    Grass,
    /// This block's ground is sand.
    Sand,
    /// This block's ground is rock.
    Rock,
}

impl Ground {
    /// Renders this ground type on the screen.
    pub fn render(
        &self,
        pos: Coord2<Nat>,
        camera: Camera,
        screen: &mut terminal::Screen,
    ) {
        if let Some(pos) = camera.convert(pos) {
            let fg = screen.get(pos).clone().fg();
            let bg = match self {
                Ground::Grass => Color::LightGreen,
                Ground::Sand => Color::LightYellow,
                Ground::Rock => Color::DarkYellow,
            };
            screen.set(pos, fg.make_tile(bg));
        }
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
    type Output = Ground;

    fn process(&self, input: I, gen: &NoiseGen) -> Self::Output {
        let index = self.weighted.process(input, gen);
        let (ground, _) = &WEIGHTS[index];
        ground.clone()
    }
}

/// A persitent map of ground types.
#[derive(Debug, Clone)]
pub struct Map {
    tree: sled::Tree,
    noise_gen: NoiseGen,
    noise_proc: FromNoise,
}

impl Map {
    /// Creates a new map given a tree that stores ground types using coordinate
    /// pairs as keys. A seed is provided to create the noise function.
    pub fn new(db: &sled::Db, seed: Seed) -> Result<Self> {
        let tree = task::block_in_place(|| db.open_tree("ground::Map"))?;
        Ok(Self {
            tree,
            noise_gen: {
                let mut noise = seed.make_noise_gen::<_, StdRng>(SEED_SALT);
                noise.sensitivity = 0.00075;
                noise
            },
            noise_proc: FromNoise::new(),
        })
    }

    /// Sets a ground type at a given point.
    pub async fn set(&self, point: Coord2<Nat>, value: &Ground) -> Result<()> {
        let point_vec = save::encode(point)?;
        let ground_vec = save::encode(value)?;
        task::block_in_place(|| self.tree.insert(point_vec, ground_vec))?;
        Ok(())
    }

    /// Gets a ground type at a given point.
    pub async fn get(&self, point: Coord2<Nat>) -> Result<Ground> {
        let point_vec = save::encode(point)?;
        let res = task::block_in_place(|| self.tree.get(point_vec));

        match res? {
            Some(bytes) => Ok(save::decode(&bytes)?),
            None => {
                let ground = self.noise_proc.process(point, &self.noise_gen);
                self.set(point, &ground).await?;
                Ok(ground)
            },
        }
    }
}
