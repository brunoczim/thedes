use crate::{
    coord::{Coord2, Direc, Nat},
    error::Result,
    matter::block,
    rand::{NoiseGen, NoiseInput, NoiseProcessor, Seed, WeightedNoise},
    storage::save,
    structures::HouseGenConfig,
};
use rand::rngs::StdRng;
use std::{collections::HashSet, fmt};
use tokio::task;

const SEED_SALT: u128 = 0xD0B8F873AB476BF213B570C3284608A3;

const MIN_HOUSES: u16 = 2;
const HOUSE_MIN_DENSITY_NUM: u16 = 1;
const HOUSE_MIN_DENSITY_DEN: u16 = 3;
const HOUSE_MAX_DENSITY_NUM: u16 = 5;
const HOUSE_MAX_DENSITY_DEN: u16 = 6;
const HOUSE_ATTEMPTS_NUM: u16 = 2;
const HOUSE_ATTEMPTS_DEN: u16 = 1;
const HOUSE_MIN_DISTANCE: u16 = 3;
const HOUSE_MIN_SIZE: u16 = 5;
const HOUSE_MAX_SIZE: u16 = 20;

const WEIGHTS: &'static [(bool, u64)] = &[(false, 2), (true, 1)];

/// ID of a thede.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Id(u16);

impl fmt::Display for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, fmt)
    }
}

fn dummy_id() -> Id {
    Id(0)
}

/// A thede's data.
#[derive(
    Debug, Clone, Copy, PartialEq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Thede {
    #[serde(skip)]
    #[serde(default = "dummy_id")]
    id: Id,
}

/// Storage registry for thedes.
#[derive(Debug, Clone)]
pub struct Registry {
    tree: sled::Tree,
}

impl Registry {
    /// Attempts to open the tree of the thedes' registry.
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = task::block_in_place(|| db.open_tree("thede::Registry"))?;
        Ok(Self { tree })
    }

    /// Registers a new thede and returns it ID.
    pub async fn register(
        &self,
        db: &sled::Db,
        map: &Map,
        blocks: &block::Map,
        seed: Seed,
        start_point: Coord2<Nat>,
    ) -> Result<Id> {
        let fut = save::generate_id(
            db,
            &self.tree,
            |id| Id(id as u16),
            |&id| async move {
                let points = map.explore(id, start_point).await?;
                self.generate_structures(points, blocks, seed).await?;
                Ok(Thede { id })
            },
        );
        fut.await
    }

    async fn generate_structures(
        &self,
        points: Vec<Coord2<Nat>>,
        blocks: &block::Map,
        seed: Seed,
    ) -> Result<()> {
        let rng = seed.make_rng::<_, StdRng>(&points);

        let max_house = HOUSE_MAX_SIZE + HOUSE_MIN_DISTANCE;
        let min_density_den = max_house * max_house * HOUSE_MIN_DENSITY_DEN;
        let max_density_den = max_house * max_house * HOUSE_MAX_DENSITY_DEN;
        let attempts_den = max_house * max_house * HOUSE_ATTEMPTS_DEN;
        let len = points.len() as Nat;
        let min_houses = len / min_density_den * HOUSE_MIN_DENSITY_NUM;
        let min_houses = min_houses.max(MIN_HOUSES);
        let max_houses = len / max_density_den * HOUSE_MAX_DENSITY_NUM;
        let max_houses = max_houses.max(MIN_HOUSES + 1);
        let attempts = len / attempts_den * HOUSE_ATTEMPTS_NUM;
        let attempts = attempts.max(max_houses);

        let gen_houses = HouseGenConfig {
            points,
            min_size: Coord2::from_axes(|_| HOUSE_MIN_SIZE),
            max_size: Coord2::from_axes(|_| HOUSE_MAX_SIZE),
            min_distance: HOUSE_MIN_DISTANCE,
            min_houses,
            max_houses,
            attempts,
            rng,
        };

        for house in gen_houses {
            house.spawn(blocks).await?;
        }

        Ok(())
    }
}

/// Map storage of thedes.
#[derive(Debug, Clone)]
pub struct Map {
    tree: sled::Tree,
    noise_gen: NoiseGen,
    noise_proc: FromNoise,
}

impl Map {
    /// Attempts to open the tree of the thedes' map.
    pub async fn new(db: &sled::Db, seed: Seed) -> Result<Self> {
        let tree = task::block_in_place(|| db.open_tree("thede::Map"))?;
        let mut noise_gen = seed.make_noise_gen::<_, StdRng>(SEED_SALT);
        noise_gen.sensitivity = 0.005;
        Ok(Self { tree, noise_gen, noise_proc: FromNoise::new() })
    }

    /// Gets the ID of a thede owner of a given point.
    pub async fn get(
        &self,
        point: Coord2<Nat>,
        registry: &Registry,
        db: &sled::Db,
        blocks: &block::Map,
        seed: Seed,
    ) -> Result<Option<Id>> {
        let point_bytes = save::encode(point)?;
        match task::block_in_place(|| self.tree.get(point_bytes))? {
            Some(bytes) => Ok(save::decode(&bytes)?),
            None => {
                let has_thede = self.noise_proc.process(point, &self.noise_gen);
                if has_thede {
                    let id = registry
                        .register(db, self, blocks, seed, point)
                        .await?;
                    Ok(Some(id))
                } else {
                    self.set(point, None).await?;
                    Ok(None)
                }
            },
        }
    }

    /// Sets the ID of a thede as the owner of a given point.
    pub async fn set(&self, point: Coord2<Nat>, id: Option<Id>) -> Result<()> {
        let point_bytes = save::encode(point)?;
        let id_bytes = save::encode(id)?;
        self.tree.insert(point_bytes, id_bytes)?;
        Ok(())
    }

    // test b1ad4a6ab6bb806d
    async fn explore(
        &self,
        id: Id,
        point: Coord2<Nat>,
    ) -> Result<Vec<Coord2<Nat>>> {
        let mut stack = vec![point];
        let mut visited = HashSet::new();

        while let Some(point) = stack.pop() {
            if visited.insert(point) {
                self.set(point, Some(id)).await?;
                for direc in Direc::iter() {
                    if let Some(new_point) = point.move_by_direc(direc) {
                        let has_thede =
                            self.noise_proc.process(new_point, &self.noise_gen);
                        let point_bytes = save::encode(new_point)?;
                        let in_place = task::block_in_place(|| {
                            self.tree.get(point_bytes)
                        })?;
                        if has_thede && in_place.is_none() {
                            stack.push(new_point);
                        }
                    }
                }
            }
        }

        let mut vec = visited.into_iter().collect::<Vec<_>>();
        vec.sort();
        Ok(vec)
    }
}

/// A type that computes from noise whether there is a thede.
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
    type Output = bool;

    fn process(&self, input: I, gen: &NoiseGen) -> Self::Output {
        let index = self.weighted.process(input, gen);
        let (has_thede, _) = &WEIGHTS[index];
        *has_thede
    }
}
