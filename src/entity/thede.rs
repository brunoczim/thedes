use crate::{
    entity::{
        language::{Language, Meaning},
        npc,
    },
    error::Result,
    math::{
        plane::{Coord2, Direc, Nat},
        rand::{
            noise::{NoiseGen, NoiseProcessor},
            weighted,
            Seed,
        },
    },
    matter::block,
    storage::save,
    structures::HouseGenConfig,
};
use ahash::AHasher;
use num::rational::Ratio;
use rand::rngs::StdRng;
use std::{
    collections::HashSet,
    error::Error,
    fmt,
    hash::{Hash, Hasher},
};
use tokio::task;

const SEED_SALT: u64 = 0x13B570C3284608A3;

type Weight = u64;

const HOUSE_MIN_ATTEMPTS: Nat = 2;
const HOUSE_ATTEMPTS: Ratio<Nat> = Ratio::new_raw(5, 2);
const HOUSE_MIN_DISTANCE: Nat = 3;
const HOUSE_MIN_SIZE: Nat = 5;
const HOUSE_MAX_SIZE: Nat = 20;

const WEIGHTS: &'static [weighted::Entry<bool, Weight>] = &[
    weighted::Entry { data: false, weight: 2 },
    weighted::Entry { data: true, weight: 1 },
];

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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Thede {
    #[serde(skip)]
    #[serde(default = "dummy_id")]
    id: Id,
    hash: u64,
    language: Language,
}

impl Thede {
    /// Returns a reference to the language.
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Returns a mutable reference to the language.
    pub fn language_mut(&mut self) -> &mut Language {
        &mut self.language
    }
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
        npcs: &npc::Registry,
        seed: Seed,
        start_point: Coord2<Nat>,
    ) -> Result<Id> {
        let fut = save::generate_id(
            db,
            &self.tree,
            |id| Id(id as u16),
            |&id| async move {
                let exploration = map.explore(id, start_point).await?;
                let hash = exploration.hash;
                let fut = self.generate_structures(
                    exploration,
                    db,
                    blocks,
                    npcs,
                    id,
                    seed,
                );
                fut.await?;

                let mut language = Language::random(seed, hash);
                for &meaning in Meaning::ALL {
                    language.gen_word(meaning, seed);
                }

                Ok(Thede { id, hash, language })
            },
        );
        fut.await
    }

    /// Loads a thede. If not found, error is returned.
    pub async fn load(&self, id: Id) -> Result<Thede> {
        let id_bytes = save::encode(id)?;
        let bytes = self.tree.get(id_bytes)?.ok_or(InvalidId(id))?;
        let thede = save::decode(&bytes)?;
        Ok(thede)
    }

    async fn generate_structures(
        &self,
        exploration: Exploration,
        db: &sled::Db,
        blocks: &block::Map,
        npcs: &npc::Registry,
        id: Id,
        seed: Seed,
    ) -> Result<()> {
        let rng = seed.make_rng::<_, StdRng>(exploration.hash);

        let max_house = HOUSE_MAX_SIZE + HOUSE_MIN_DISTANCE;
        let attempts_den = max_house * max_house * HOUSE_ATTEMPTS.denom();
        let len = exploration.points.len() as Nat;
        let attempts = len / attempts_den * HOUSE_ATTEMPTS.numer();
        let attempts = attempts.max(HOUSE_MIN_ATTEMPTS);

        let gen_houses = HouseGenConfig {
            points: exploration.points,
            min_size: Coord2::from_axes(|_| HOUSE_MIN_SIZE),
            max_size: Coord2::from_axes(|_| HOUSE_MAX_SIZE),
            min_distance: HOUSE_MIN_DISTANCE,
            attempts,
            rng,
        };

        for house in gen_houses {
            let head = house.rect.start.map(|a| a + 1);
            let facing = Direc::Down;
            npcs.register(db, blocks, head, facing, id).await?;
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
    noise_proc: weighted::Entries<bool, Weight>,
}

impl Map {
    /// Attempts to open the tree of the thedes' map.
    pub async fn new(db: &sled::Db, seed: Seed) -> Result<Self> {
        let tree = task::block_in_place(|| db.open_tree("thede::Map"))?;
        let mut noise_gen = seed.make_noise_gen::<_, StdRng>(SEED_SALT);
        noise_gen.sensitivity = 0.005;
        let noise_proc = weighted::Entries::new(WEIGHTS.iter().cloned());
        Ok(Self { tree, noise_gen, noise_proc })
    }

    /// Gets the ID of a thede owner of a given point.
    pub async fn get(
        &self,
        point: Coord2<Nat>,
        registry: &Registry,
        db: &sled::Db,
        blocks: &block::Map,
        npcs: &npc::Registry,
        seed: Seed,
    ) -> Result<Option<Id>> {
        let point_bytes = save::encode(point)?;
        match task::block_in_place(|| self.tree.get(point_bytes))? {
            Some(bytes) => Ok(save::decode(&bytes)?),
            None => {
                let has_thede =
                    (&&self.noise_proc).process(point, &self.noise_gen).data;
                if has_thede {
                    let id = registry
                        .register(db, self, blocks, npcs, seed, point)
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
    async fn explore(&self, id: Id, point: Coord2<Nat>) -> Result<Exploration> {
        let mut stack = vec![point];
        let mut visited = HashSet::new();
        let mut hasher = AHasher::new_with_keys(0, 0);

        while let Some(point) = stack.pop() {
            if visited.insert(point) {
                point.hash(&mut hasher);
                self.set(point, Some(id)).await?;
                for direc in Direc::iter() {
                    if let Some(new_point) = point.move_by_direc(direc) {
                        let has_thede = (&&self.noise_proc)
                            .process(new_point, &self.noise_gen)
                            .data;
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

        let mut points = visited.into_iter().collect::<Vec<_>>();
        points.sort();
        Ok(Exploration { points, hash: hasher.finish() })
    }
}

#[derive(Debug)]
struct Exploration {
    hash: u64,
    points: Vec<Coord2<Nat>>,
}

/// Returned by [`Registry::load`] if the player does not exist.
#[derive(Debug, Clone, Copy)]
pub struct InvalidId(pub Id);

impl fmt::Display for InvalidId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Invalid thede id {}", self.0)
    }
}

impl Error for InvalidId {}
