use crate::{
    coord::{Coord2, Direc, Nat, Rect},
    error::Result,
    matter::block,
    rand::{NoiseGen, NoiseInput, NoiseProcessor, Seed, WeightedNoise},
    storage::save,
    structures::{RectDistribution, RectHouse},
};
use rand::{rngs::StdRng, Rng};
use std::{collections::HashSet, fmt};
use tokio::task;

const SEED_SALT: u128 = 0xD0B8F873AB476BF213B570C3284608A3;

const MIN_HOUSES: u16 = 2;
const HOUSE_DENSITY_DEN: u16 = 3;
const HOUSE_MIN_DISTANCE: u16 = 3;

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
                let rect = map.explore(id, start_point).await?;
                self.generate_structures(rect, map, blocks, id, seed).await?;
                Ok(Thede { id })
            },
        );
        fut.await
    }

    async fn generate_structures(
        &self,
        rect: Rect,
        map: &Map,
        blocks: &block::Map,
        id: Id,
        seed: Seed,
    ) -> Result<()> {
        let mut rng = seed.make_rng::<_, StdRng>(rect);

        let distribution = RectDistribution {
            low_limit: rect.start,
            high_limit: rect.end(),
            min_size: Coord2 { x: 4, y: 4 },
            max_size: Coord2 { x: 20, y: 20 },
        };

        let mut generated = Vec::<RectHouse>::new();
        let mean_dim = rect.size.foldl(0, |a, b| a + b / 2 + b % 2);
        let attempts =
            rect.size.x * rect.size.y / (mean_dim * HOUSE_DENSITY_DEN);
        let attempts = attempts.max(MIN_HOUSES);
        let min_success = rng.gen_range(MIN_HOUSES, attempts + 1);

        let mut i = 0;

        while (generated.len() as Nat) < min_success || i < attempts {
            i += 1;

            let new_house = rng.sample(&distribution);
            let expanded_rect = Rect {
                start: new_house
                    .rect
                    .start
                    .map(|a| a.saturating_sub(HOUSE_MIN_DISTANCE)),
                size: new_house
                    .rect
                    .size
                    .map(|a| a.saturating_add(HOUSE_MIN_DISTANCE * 2)),
            };
            let overlaps = generated
                .iter()
                .any(|house| expanded_rect.overlaps(house.rect));
            if !overlaps {
                let mut inside = true;

                for point in new_house.rect.lines() {
                    let at_point = map.get_raw(point).await?;
                    if at_point != Some(id) {
                        inside = false;
                        break;
                    }
                }

                if inside {
                    new_house.spawn(blocks).await?;
                    generated.push(new_house);
                }
            }
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

    async fn get_raw(&self, point: Coord2<Nat>) -> Result<Option<Id>> {
        let point_bytes = save::encode(point)?;
        match task::block_in_place(|| self.tree.get(point_bytes))? {
            Some(bytes) => Ok(save::decode(&bytes)?),
            None => Ok(None),
        }
    }

    async fn explore(&self, id: Id, point: Coord2<Nat>) -> Result<Rect> {
        let mut stack = vec![point];
        let mut north = point.y;
        let mut south = point.y;
        let mut west = point.x;
        let mut east = point.x;
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
                            north = north.min(new_point.y);
                            south = south.max(new_point.y);
                            west = west.min(new_point.x);
                            east = east.max(new_point.x);
                            stack.push(new_point);
                        }
                    }
                }
            }
        }

        let rect = Rect {
            start: Coord2 { x: west, y: north },
            size: Coord2 { x: east - west + 1, y: south - north + 1 },
        };

        Ok(rect)
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
