use crate::{
    entity::language::{Language, Meaning},
    error::Result,
    map::Coord,
    math::rand::{
        noise::{NoiseGen, NoiseProcessor},
        weighted,
        Seed,
    },
    storage::save::SavedGame,
    structures::{Village, VillageGenConfig},
};
use ahash::AHasher;
use gardiz::{coord::Vec2, direc::Direction, set::Set};
use kopidaz::tree::Tree;
use num::{integer, rational::Ratio};
use rand::rngs::StdRng;
use std::{
    error::Error,
    fmt,
    hash::{Hash, Hasher},
};
use tracing::Instrument;

const SEED_SALT: u64 = 0x13B570C3284608A3;

type Weight = u64;

const MIN_HOUSES: Coord = 2;
const VERTEX_DISTANCING: Coord = 5;
const MIN_VERTEX_ATTEMPTS: Coord = 3;
const MAX_VERTEX_ATTEMPTS_RATIO: Ratio<Coord> = Ratio::new_raw(5, 4);
const MIN_EDGE_ATTEMPTS: Coord = 1;
const MAX_EDGE_ATTEMPTS_RATIO: Ratio<Coord> = Ratio::new_raw(11, 3);
const MIN_HOUSE_ATTEMPTS: Coord = MIN_HOUSES + 1;
const MAX_HOUSE_ATTEMPTS_RATIO: Ratio<Coord> = Ratio::new_raw(3, 2);
const MIN_HOUSE_SIZE: Coord = 5;
const MAX_HOUSE_SIZE: Coord = 15;

const EXPLORE_STACK_CAPACITY: usize = 0x8000;

const WEIGHTS: &'static [weighted::Entry<bool, Weight>] = &[
    weighted::Entry { data: false, weight: 5 },
    weighted::Entry { data: true, weight: 3 },
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
    tree: Tree<Id, Thede>,
}

impl Registry {
    /// Attempts to open the tree of the thedes' registry.
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = Tree::open(db, "thede::Registry").await?;
        Ok(Self { tree })
    }

    pub async fn register(
        &self,
        start: Vec2<Coord>,
        game: &SavedGame,
        generator: &Generator,
    ) -> Result<Option<Id>> {
        let exploration = generator.explore(start)?;
        let hash = exploration.hash;
        let village = generator
            .gen_structures(exploration.clone(), game.seed())
            .instrument(tracing::debug_span!("gen_structures"))
            .await?;

        let language = tracing::debug_span!("Language").in_scope(|| {
            let mut language = Language::random(game.seed(), hash);
            for &meaning in Meaning::ALL {
                language.gen_word(meaning, game.seed(), hash);
            }
            language
        });

        if village.houses.len() >= MIN_HOUSES as usize {
            let future = self.tree.generate_id(
                game.db(),
                |id| async move { Result::Ok(Id(id as u16)) },
                |&id| async move { Ok(Thede { id, hash, language }) },
            );

            let id = future.await?;
            generator.spawn(&village, id, game, &exploration).await?;

            Ok(Some(id))
        } else {
            generator.abort(game, &exploration).await?;
            Ok(None)
        }
    }

    /// Loads a thede. If not found, error is returned.
    pub async fn load(&self, id: Id) -> Result<Thede> {
        let thede = self.tree.get(&id).await?.ok_or(InvalidId(id))?;
        Ok(thede)
    }
}

#[derive(Debug, Clone)]
struct Exploration {
    hash: u64,
    area: Set<Coord>,
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

/// A weighted generator of thedes.
#[derive(Debug, Clone)]
pub struct Generator {
    noise_gen: NoiseGen,
    processor: weighted::Entries<bool, Weight>,
}

impl Generator {
    /// Creates a new generator.
    pub fn new(seed: Seed) -> Generator {
        let mut noise_gen = seed.make_noise_gen::<_, StdRng>(SEED_SALT);
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
        game: &SavedGame,
    ) -> Result<()> {
        if self.is_thede_at(start) {
            game.thedes().register(start, game, self).await?;
        } else {
            game.map().set_thede_raw(start, MapLayer::Empty).await?;
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
        let rng = seed.make_rng::<_, StdRng>(exploration.hash);
        let len = exploration.area.len();

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

        let ideal_houses = Ratio::from(len as Coord / MAX_HOUSE_SIZE.pow(2));
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

        tracing::debug!(
            ?max_vertex_attempts,
            ?max_edge_attempts,
            ?max_house_attempts
        );

        Ok(tracing::debug_span!("gen").in_scope(|| generation.gen()))
    }

    async fn spawn(
        &self,
        village: &Village,
        id: Id,
        game: &SavedGame,
        exploration: &Exploration,
    ) -> Result<()> {
        for point in exploration.area.rows() {
            game.map()
                .set_thede_raw(point.copied(), MapLayer::Thede(id))
                .await?;
        }
        village.spawn(game).await?;

        for house in &village.houses {
            let head = house.rect.start.map(|a| a + 1);
            let facing = Direction::Down;
            game.npcs().register(game, head, facing, id).await?;
        }

        Ok(())
    }

    async fn abort(
        &self,
        game: &SavedGame,
        exploration: &Exploration,
    ) -> Result<()> {
        for point in exploration.area.rows() {
            game.map().set_thede_raw(point.copied(), MapLayer::Empty).await?;
        }
        Ok(())
    }
}

/// A map layer of thedes.
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
pub enum MapLayer {
    /// There is a thede here with this `Id`.
    Thede(Id),
    /// No thede here.
    Empty,
}

impl fmt::Display for MapLayer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MapLayer::Thede(id) => fmt::Display::fmt(id, fmt),
            MapLayer::Empty => fmt.pad("none"),
        }
    }
}
