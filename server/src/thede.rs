mod structures;

use crate::{
    language,
    map::Map,
    npc,
    random::{
        make_rng,
        noise::{NoiseGen, NoiseProcessor},
        weighted,
    },
};
use ahash::AHasher;
use gardiz::{coord::Vec2, direc::Direction, set::Set};
use kopidaz::tree::Tree;
use num::{integer, rational::Ratio};
use rand::rngs::StdRng;
use std::hash::{Hash, Hasher};
use structures::{Village, VillageGenConfig};
use thedes_common::{
    error::{BadThedeId, Error},
    map::Coord,
    seed::Seed,
    Result,
    ResultExt,
};

pub use thedes_common::thede::{Id, MapData};

const SEED_SALT: u64 = 0x13B570C3284608A3;

type Weight = u64;

const MIN_HOUSES: Coord = 2;
const VERTEX_DISTANCING: Coord = 32;
const MIN_VERTEX_ATTEMPTS: Coord = 3;
const MAX_VERTEX_ATTEMPTS_RATIO: Ratio<Coord> = Ratio::new_raw(2, 7);
const MIN_EDGE_ATTEMPTS: Coord = 1;
const MAX_EDGE_ATTEMPTS_RATIO: Ratio<Coord> = Ratio::new_raw(2, 11);
const MIN_HOUSE_ATTEMPTS: Coord = MIN_HOUSES + 1;
const MAX_HOUSE_ATTEMPTS_RATIO: Ratio<Coord> = Ratio::new_raw(3, 2);
const MIN_HOUSE_SIZE: Coord = 5;
const MAX_HOUSE_SIZE: Coord = 15;

const EXPLORE_STACK_CAPACITY: usize = 0x8000;

const WEIGHTS: &'static [weighted::Entry<bool, Weight>] = &[
    weighted::Entry { data: false, weight: 5 },
    weighted::Entry { data: true, weight: 3 },
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Thede {
    pub id: Id,
    pub data: Data,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Data {
    pub hash: u64,
    pub language: language::Id,
}

/// Storage registry for thedes.
#[derive(Debug, Clone)]
pub struct Registry {
    tree: Tree<Id, Data>,
}

impl Registry {
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = Tree::open(db, "thede::Registry").await.erase_err()?;
        Ok(Self { tree })
    }

    pub async fn register(
        &self,
        start: Vec2<Coord>,
        generator: &Generator,
        db: &sled::Db,
        seed: Seed,
        npcs: &npc::Registry,
        languages: &language::Registry,
        map: &mut Map,
    ) -> Result<Option<Thede>> {
        let exploration = generator.explore(start)?;
        let hash = exploration.hash;
        let village =
            generator.gen_structures(exploration.clone(), seed).await?;

        let language = languages.register(db, seed, hash).await?;

        if village.houses.len() >= MIN_HOUSES as usize {
            let (id, data) = self
                .tree
                .id_builder()
                .error_conversor(Error::erase)
                .id_maker(|bits| Id(bits as _))
                .data_maker(|_| Data { hash, language: language.id })
                .generate(db)
                .await?;

            generator.spawn(&village, id, &exploration, db, npcs, map).await?;

            Ok(Some(Thede { id, data }))
        } else {
            generator.abort(map, &exploration).await?;
            Ok(None)
        }
    }

    /// Loads a thede. If not found, error is returned.
    pub async fn load(&self, id: Id) -> Result<Thede> {
        let data =
            self.tree.get(&id).await.erase_err()?.ok_or(BadThedeId { id })?;
        Ok(Thede { id, data })
    }
}

#[derive(Debug, Clone)]
struct Exploration {
    hash: u64,
    area: Set<Coord>,
}

/// A weighted generator of thedes.
#[derive(Debug, Clone)]
pub struct Generator {
    noise_gen: NoiseGen,
    processor: weighted::Entries<bool, Weight>,
}

impl Generator {
    /// Creates a new generator.
    pub fn new(seed: Seed) -> Generator {
        let mut noise_gen = NoiseGen::new::<_, StdRng>(seed, SEED_SALT);
        noise_gen.sensitivity = 0.003;
        let processor = weighted::Entries::new(WEIGHTS.iter().cloned());
        Self { noise_gen, processor }
    }

    /// Generates whether thede should be a thede at a given location.
    pub fn is_thede_at(&self, point: Vec2<Coord>) -> bool {
        (&&self.processor).process(point, &self.noise_gen).data
    }

    /// Generates a thede starting from the `start` point. If no thede should be
    /// present, the point is initialized to `MapLayer::Empty`.
    pub async fn generate(
        &self,
        start: Vec2<Coord>,
        db: &sled::Db,
        seed: Seed,
        languages: &language::Registry,
        npcs: &npc::Registry,
        registry: &Registry,
        map: &mut Map,
    ) -> Result<()> {
        if self.is_thede_at(start) {
            registry
                .register(start, self, db, seed, npcs, languages, map)
                .await?;
        } else {
            map.entry_mut(start).await?.thede = MapData::Empty;
        }
        Ok(())
    }

    fn explore(&self, start: Vec2<Coord>) -> Result<Exploration> {
        let mut stack = Vec::with_capacity(EXPLORE_STACK_CAPACITY);
        stack.push(start);
        let mut visited = Set::new();

        while let Some(point) = stack.pop() {
            visited.insert(point);
            for direction in Direction::iter() {
                if let Some(new_point) = point
                    .checked_move(direction)
                    .filter(|&point| !visited.contains(point.as_ref()))
                {
                    let is_thede = self.is_thede_at(new_point);
                    let is_empty = !visited.contains(new_point.as_ref());
                    if is_thede && is_empty {
                        stack.push(new_point);
                    }
                }
            }
        }

        let mut hasher = AHasher::new_with_keys(0, 0);
        for coord in visited.rows() {
            coord.hash(&mut hasher);
        }
        Ok(Exploration { area: visited, hash: hasher.finish() })
    }

    // test 74b2e893324284de
    async fn gen_structures(
        &self,
        exploration: Exploration,
        seed: Seed,
    ) -> Result<Village> {
        let rng = make_rng::<_, StdRng>(seed, exploration.hash);
        let len = exploration.area.len();

        let ideal_houses = Ratio::from(len as Coord / MAX_HOUSE_SIZE.pow(2));

        let ideal_vertices = Ratio::new(
            integer::sqrt(len as Coord),
            VERTEX_DISTANCING * (VERTEX_DISTANCING - 2),
        );
        let max_vertex_attempts = (MAX_VERTEX_ATTEMPTS_RATIO * ideal_vertices)
            .to_integer()
            .max(MIN_VERTEX_ATTEMPTS);

        // Formula for maximum edges in planar graph - potentially existing
        // vertices.
        //
        // Formula: e = 3v - 6
        let ideal_edges = (3 * max_vertex_attempts)
            .saturating_sub(6)
            .saturating_sub(max_vertex_attempts);
        let max_edge_attempts = (MAX_EDGE_ATTEMPTS_RATIO * ideal_edges)
            .to_integer()
            .max(MIN_EDGE_ATTEMPTS);

        let max_house_attempts = (MAX_HOUSE_ATTEMPTS_RATIO * ideal_houses)
            .to_integer()
            .max(MIN_HOUSE_ATTEMPTS);

        let generation = VillageGenConfig {
            area: exploration.area,
            min_vertex_attempts: MIN_VERTEX_ATTEMPTS,
            max_vertex_attempts,
            min_edge_attempts: MIN_EDGE_ATTEMPTS,
            max_edge_attempts,
            min_house_attempts: MIN_HOUSE_ATTEMPTS,
            max_house_attempts,
            min_house_size: Vec2::from_axes(|_| MIN_HOUSE_SIZE),
            max_house_size: Vec2::from_axes(|_| MAX_HOUSE_SIZE),
            rng,
        };

        Ok(generation.gen())
    }

    async fn spawn(
        &self,
        village: &Village,
        id: Id,
        exploration: &Exploration,
        db: &sled::Db,
        npcs: &npc::Registry,
        map: &mut Map,
    ) -> Result<()> {
        for point in exploration.area.rows() {
            map.entry_mut(point.copied()).await?.thede = MapData::Thede(id);
        }
        village.spawn(map).await?;

        for house in &village.houses {
            let head = house.rect.start.map(|a| a + 1);
            let facing = Direction::Down;
            npcs.register(head, facing, id, db, map).await?;
        }

        Ok(())
    }

    async fn abort(
        &self,
        map: &mut Map,
        exploration: &Exploration,
    ) -> Result<()> {
        for point in exploration.area.rows() {
            map.entry_mut(point.copied()).await?.thede = MapData::Empty;
        }
        Ok(())
    }
}
