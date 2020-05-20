use crate::{
    entity::{biome, thede, Biome},
    error::Result,
    math::{
        plane::{Coord2, Nat, Rect},
        rand::Seed,
    },
    matter::{Block, Ground},
    storage::save::{SavedGame, Tree},
};
use ndarray::{Array, Ix, Ix2};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};
use tokio::sync::{Mutex, MutexGuard};

const CHUNK_SIZE_EXP: Coord2<Nat> = Coord2 { x: 4, y: 4 };
const CHUNK_SIZE: Coord2<Nat> =
    Coord2 { x: 1 << CHUNK_SIZE_EXP.x, y: 1 << CHUNK_SIZE_EXP.y };
const CHUNK_SHAPE: [Ix; 2] = [CHUNK_SIZE.y as usize, CHUNK_SIZE.x as usize];
const MIN_CACHE_LIMIT: usize = 4;

pub const RECOMMENDED_CACHE_LIMIT: usize = 64;

fn layer_must_be_set() -> ! {
    panic!("Map layer required to be set, but it isn't")
}

fn layer_must_not_be_gening() -> ! {
    panic!("Map layer required to not being generating, but it is")
}

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
pub enum RawLayer<T> {
    Set(T),
    Generating,
    NotGenerated,
}

impl<T> RawLayer<T> {
    pub fn as_ref(&self) -> RawLayer<&T> {
        match self {
            RawLayer::Set(val) => RawLayer::Set(val),
            RawLayer::Generating => RawLayer::Generating,
            RawLayer::NotGenerated => RawLayer::NotGenerated,
        }
    }

    pub fn as_mut(&mut self) -> RawLayer<&mut T> {
        match self {
            RawLayer::Set(val) => RawLayer::Set(val),
            RawLayer::Generating => RawLayer::Generating,
            RawLayer::NotGenerated => RawLayer::NotGenerated,
        }
    }

    pub fn must_be_set(self) -> T {
        match self {
            RawLayer::Set(val) => val,
            _ => layer_must_be_set(),
        }
    }

    pub fn must_not_be_gening(self) -> Option<T> {
        match self {
            RawLayer::Set(val) => Some(val),
            RawLayer::NotGenerated => None,
            RawLayer::Generating => layer_must_not_be_gening(),
        }
    }

    pub fn to_set(self) -> Option<T> {
        match self {
            RawLayer::Set(val) => Some(val),
            RawLayer::NotGenerated | RawLayer::Generating => None,
        }
    }
}

impl<T> Default for RawLayer<T> {
    fn default() -> Self {
        RawLayer::NotGenerated
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    inner: Arc<Mutex<MapInner>>,
}

impl Map {
    pub async fn new(
        db: &sled::Db,
        seed: Seed,
        cache_limit: usize,
    ) -> Result<Self> {
        let inner = MapInner {
            cache: Cache::new(cache_limit),
            tree: Tree::open(db, "Map"),
            biome_gen: Arc::new(biome::Generator::new(seed)),
            thede_gen: Arc::new(thede::Generator::new(seed)),
        };
        Self { inner: Arc::new(Mutex::new(inner)) }
    }

    pub async fn biome_raw(
        &self,
        point: Coord2<Nat>,
    ) -> Result<RawLayer<Biome>> {
        let ret = self.lock().entry(point).await?.clone();
        Ok(ret)
    }

    pub async fn biome(&self, point: Coord2<Nat>) -> Result<Biome> {
        let ret = self.lock().biome(point).await?.clone();
        Ok(ret)
    }

    pub async fn set_biome(
        &self,
        point: Coord2<Nat>,
        biome: Biome,
    ) -> Result<()> {
        *self.lock().biome(point).await? = biome;
        Ok(())
    }

    pub async fn ground_raw(
        &self,
        point: Coord2<Nat>,
    ) -> Result<RawLayer<Ground>> {
        let ret = self.lock().entry(point).await?.clone();
        Ok(ret)
    }

    pub async fn ground(&self, point: Coord2<Nat>) -> Result<Ground> {
        let ret = self.lock().ground(point).await?.clone();
        Ok(ret)
    }

    pub async fn set_ground(
        &self,
        point: Coord2<Nat>,
        ground: Ground,
    ) -> Result<()> {
        *self.lock().ground(point).await? = ground;
        Ok(())
    }

    pub async fn block_raw(
        &self,
        point: Coord2<Nat>,
    ) -> Result<RawLayer<Block>> {
        let ret = self.lock().entry(point).await?.clone();
        Ok(ret)
    }

    pub async fn block(&self, point: Coord2<Nat>) -> Result<Block> {
        let ret = self.lock().block(point).await?.clone();
        Ok(ret)
    }

    pub async fn set_block(
        &self,
        point: Coord2<Nat>,
        block: Block,
    ) -> Result<()> {
        *self.lock().block(point).await? = block;
        Ok(())
    }

    pub async fn thede_raw(
        &self,
        point: Coord2<Nat>,
    ) -> Result<RawLayer<thede::MapLayer>> {
        let ret = self.lock().entry(point).await?.clone();
        Ok(ret)
    }

    pub async fn thede(&self, point: Coord2<Nat>) -> Result<thede::MapLayer> {
        let ret = self.lock().thede(point).await?.clone();
        Ok(ret)
    }

    pub async fn set_thede(
        &self,
        point: Coord2<Nat>,
        thede: thede::MapLayer,
    ) -> Result<()> {
        *self.lock().thede(point).await? = thede;
        Ok(())
    }

    async fn lock<'map>(&'map self) -> LockedMap<'map> {
        LockedMap { guard: None, map: self }
    }
}

#[derive(Debug, Clone)]
struct MapInner {
    cache: Cache,
    tree: Tree<Coord2<Nat>, Chunk>,
    biome_gen: Arc<biome::Generator>,
    thede_gen: Arc<thede::Generator>,
}

#[derive(Debug)]
struct LockedMap<'map> {
    guard: Option<MutexGuard<'map, MapInner>>,
    map: &'map Map,
}

impl<'map> LockedMap<'map> {
    async fn chunk(&mut self, index: Coord2<Nat>) -> Result<&mut Chunk> {
        self.require_chunk(index).await?;
        Ok(self.inner().cache.chunk_mut(index).expect("I just loaded it"))
    }

    async fn entry(&mut self, point: Coord2<Nat>) -> Result<&mut Entry> {
        let index = unpack_chunk(point);
        self.require_chunk(index).await?;
        Ok(self.inner().cache.entry_mut(point).expect("I just loaded it"))
    }

    async fn biome(&mut self, point: Coord2<Nat>) -> Result<&mut Biome> {
        let needs_gen = self
            .entry(point)
            .await?
            .entry
            .biome
            .as_mut()
            .must_not_be_gening()
            .is_none();

        if needs_gen {
            let biome = self.inner().biome_gen.biome_at(point);
            self.entry(point).await?.biome = RawLayer::Set(biome);
        }

        let biome = self.entry(point).await?.biome.as_mut().must_be_set();
        Ok(biome)
    }

    async fn ground(&mut self, point: Coord2<Nat>) -> Result<&mut Ground> {
        let needs_gen = self
            .entry(point)
            .await?
            .entry
            .ground
            .as_mut()
            .must_not_be_gening()
            .is_none();

        if needs_gen {
            let ground = self.biome(point).await?.main_ground();
            self.entry(point).await?.ground = RawLayer::Set(ground);
        }

        let ground = self.entry(point).await?.ground.as_mut().must_be_set();
        Ok(ground)
    }

    async fn block(&mut self, point: Coord2<Nat>) -> Result<&mut Block> {
        let needs_gen = self
            .entry(point)
            .await?
            .entry
            .block
            .as_mut()
            .must_not_be_gening()
            .is_none();

        if needs_gen {
            self.entry(point).await?.block = RawLayer::Set(Block::Empty);
        }

        let block = self.entry(point).await?.block.as_mut().must_be_set();
        Ok(block)
    }

    async fn thede(
        &mut self,
        point: Coord2<Nat>,
        game: &SavedGame,
    ) -> Result<&mut thede::MapLayer> {
        let needs_gen = self
            .entry(point)
            .await?
            .entry
            .thede
            .as_mut()
            .must_not_be_gening()
            .is_none();

        if needs_gen {
            self.entry(point).await?.entry.thede = RawLayer::Generating;
            let thede_gen = self.inner().thede_gen.clone();
            self.guard = None;
            thede_gen.gen(game).await?;
        }

        let thede = self.entry(point).await?.thede.as_mut().must_be_set();
        Ok(thede)
    }

    async fn inner(&mut self) -> &mut MapInner {
        if self.guard.is_none() {
            self.guard = Some(self.map.inner.lock().await);
        }

        &mut self.guard.as_mut().expect("I checked it")
    }

    async fn load_chunk(&mut self, index: Coord2<Nat>) -> Result<bool> {
        if self.inner().cache.chunk(index).is_some() {
            Ok(true)
        } else if let Some(chunk) = self.inner().tree.get(&index).await? {
            if let Some((index, chunk)) = self.inner().cache.load(index, chunk)
            {
                self.inner().tree.insert(&index, &chunk).await?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn require_chunk(&mut self, index: Coord2<Nat>) -> Result<()> {
        if !self.load_chunk(index).await? {
            self.init_chunk(index).await?;
        }
    }

    async fn init_chunk(&mut self, index: Coord2<Nat>) -> Result<()> {
        let chunk = Chunk::default();
        let inner = self.inner();
        inner.tree.insert(&index, &chunk).await?;
        if let Some((index, chunk)) = inner.cache.load(index, chunk) {
            inner.tree.insert(&index, &chunk).await?;
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
pub struct Entry {
    pub biome: RawLayer<Biome>,
    pub ground: RawLayer<Ground>,
    pub block: RawLayer<Block>,
    pub thede: RawLayer<thede::MapLayer>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Chunk {
    entries: Array<Entry, Ix2>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self { entries: Array::default(CHUNK_SHAPE) }
    }
}

fn unpack_chunk(point: Coord2<Nat>) -> Coord2<Nat> {
    point.zip_with(CHUNK_SIZE_EXP, |coord, exp| coord >> exp)
}

fn unpack_offset(point: Coord2<Nat>) -> Coord2<Nat> {
    point.zip_with(CHUNK_SIZE_EXP, |coord, exp| coord & ((1 << exp) - 1))
}

fn pack_point(chunk: Coord2<Nat>, offset: Coord2<Nat>) -> Coord2<Nat> {
    chunk.zip(offset).zip_with(CHUNK_SIZE_EXP, |(chunk, offset), exp| {
        chunk << exp | offset & ((1 << exp) - 1)
    })
}

#[derive(Debug, Clone)]
struct CachedChunk {
    chunk: Chunk,
    next: Option<Coord2<Nat>>,
    prev: Option<Coord2<Nat>>,
}

#[derive(Debug, Clone)]
struct Cache {
    limit: usize,
    needs_flush: HashSet<Coord2<Nat>>,
    chunks: HashMap<Coord2<Nat>, CachedChunk>,
    first: Option<Coord2<Nat>>,
    last: Option<Coord2<Nat>>,
}

impl Cache {
    fn new(limit: usize) -> Self {
        Self {
            limit: limit.max(MIN_CACHE_LIMIT),
            needs_flush: HashSet::new(),
            chunks: HashMap::with_capacity(limit),
            first: None,
            last: None,
        }
    }

    fn chunk(&mut self, index: Coord2<Nat>) -> Option<&Chunk> {
        if self.chunks.contains_key(&index) {
            self.access(index);
        }
        self.chunks.get(&index).map(|cached| &cached.chunk)
    }

    fn chunk_mut(&mut self, index: Coord2<Nat>) -> Option<&mut Chunk> {
        if self.chunks.contains_key(&index) {
            self.access(index);
            self.needs_flush.insert(index);
        }
        self.chunks.get_mut(&index).map(|cached| &mut cached.chunk)
    }

    fn entry(&mut self, point: Coord2<Nat>) -> Option<&Entry> {
        let chunk_index = unpack_chunk(point);
        let offset = unpack_offset(point);
        self.chunk(chunk_index)
            .map(|chunk| &chunk.entries[[offset.y as usize, offset.x as usize]])
    }

    fn entry_mut(&mut self, point: Coord2<Nat>) -> Option<&mut Entry> {
        let chunk_index = unpack_chunk(point);
        let offset = unpack_offset(point);
        self.chunk_mut(chunk_index).map(|chunk| {
            &mut chunk.entries[[offset.y as usize, offset.x as usize]]
        })
    }

    #[must_use]
    fn load(
        &mut self,
        chunk_index: Coord2<Nat>,
        chunk: Chunk,
    ) -> Option<(Coord2<Nat>, Chunk)> {
        let dropped = if self.chunks.len() >= self.limit {
            self.drop_oldest()
        } else {
            None
        };

        self.chunks.insert(
            chunk_index,
            CachedChunk { chunk, next: self.first, prev: None },
        );

        if let Some(first) = self.first {
            let chunk = self.chunks.get_mut(&first).expect("bad list");
            chunk.prev = Some(chunk_index);
        }
        self.first = Some(chunk_index);
        if self.last.is_none() {
            self.last = self.first;
        }

        dropped
    }

    #[must_use]
    fn drop_oldest(&mut self) -> Option<(Coord2<Nat>, Chunk)> {
        let ret = self.last.map(|last| {
            let last_prev = self.chunks.get_mut(&last).expect("bad list").prev;
            if let Some(prev) = last_prev {
                self.chunks.get_mut(&prev).expect("bad list").next = None;
            }
            if self.first == self.last {
                self.first = last_prev;
            }
            self.last = last_prev;
            (last, self.chunks.remove(&last).expect("bad list").chunk)
        });

        ret.filter(|(index, _)| self.needs_flush.remove(index))
    }

    fn access(&mut self, chunk_index: Coord2<Nat>) {
        if self.first != Some(chunk_index) {
            let (chunk_prev, chunk_next) = {
                let chunk = &self.chunks[&chunk_index];
                (chunk.prev, chunk.next)
            };

            if let Some(prev) = chunk_prev {
                self.chunks.get_mut(&prev).expect("bad list").next = chunk_next;
            }
            if let Some(next) = chunk_next {
                self.chunks.get_mut(&next).expect("bad list").prev = chunk_prev;
            } else {
                self.last = chunk_prev;
            }

            if let Some(first) = self.first {
                let refer = self.chunks.get_mut(&first).expect("bad list");
                refer.prev = Some(chunk_index);
            }
            {
                let chunk =
                    self.chunks.get_mut(&chunk_index).expect("bad list");
                chunk.prev = None;
                chunk.next = self.first;
            }
            self.first = Some(chunk_index);
            if self.last.is_none() {
                self.last = self.first;
            }
        }
    }

    #[allow(dead_code)]
    fn debug_iter(&self) -> CacheDebugIter {
        CacheDebugIter {
            cache: self,
            front_back: self
                .first
                .and_then(|front| self.last.map(|back| (front, back))),
        }
    }
}

#[derive(Debug)]
struct CacheDebugIter<'cache> {
    cache: &'cache Cache,
    front_back: Option<(Coord2<Nat>, Coord2<Nat>)>,
}

impl<'cache> Iterator for CacheDebugIter<'cache> {
    type Item = Coord2<Nat>;

    fn next(&mut self) -> Option<Self::Item> {
        let (front, back) = self.front_back?;
        self.front_back = if front == back {
            None
        } else {
            self.cache.chunks[&front].next.map(|front| (front, back))
        };
        Some(front)
    }
}

impl<'cache> DoubleEndedIterator for CacheDebugIter<'cache> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (front, back) = self.front_back?;
        self.front_back = if front == back {
            None
        } else {
            self.cache.chunks[&back].prev.map(|back| (front, back))
        };
        Some(back)
    }
}

#[cfg(test)]
mod test {
    use super::{pack_point, unpack_chunk, unpack_offset, Cache, Chunk};
    use crate::{entity::Biome, math::plane::Coord2, matter::Ground};

    #[test]
    fn pack_unpack() {
        let point1 = Coord2 { x: 4857, y: 7375 };
        assert_eq!(
            point1,
            pack_point(unpack_chunk(point1), unpack_offset(point1))
        );
    }

    #[test]
    fn cache() {
        let mut cache = Cache::new(4);
        let mut chunk1 = Chunk::default();
        chunk1.entries[[0, 0]].biome = Biome::Desert;
        let mut chunk2 = Chunk::default();
        chunk2.entries[[0, 1]].biome = Biome::Desert;
        let mut chunk3 = Chunk::default();
        chunk3.entries[[1, 1]].biome = Biome::Desert;
        let mut chunk4 = Chunk::default();
        chunk4.entries[[0, 0]].biome = Biome::RockDesert;
        let mut chunk5 = Chunk::default();
        chunk5.entries[[1, 0]].biome = Biome::RockDesert;
        // last = 1, 2, 3, 4 = first
        assert!(cache.load(Coord2 { x: 5, y: 0 }, chunk1.clone()).is_none());
        assert!(cache.load(Coord2 { x: 1, y: 0 }, chunk2.clone()).is_none());
        assert!(cache.load(Coord2 { x: 0, y: 1 }, chunk3.clone()).is_none());
        assert!(cache.load(Coord2 { x: 1, y: 1 }, chunk4.clone()).is_none());

        // last = 2, 3, 4, 5 = first
        assert!(cache.load(Coord2 { x: 2, y: 0 }, chunk5.clone()).is_none());
        // last = 3, 4, 5, 2 = first
        assert_eq!(cache.chunk(Coord2 { x: 1, y: 0 }), Some(&chunk2));
        // last = 3, 4, 2, 5 = first
        assert_eq!(cache.chunk(Coord2 { x: 2, y: 0 }), Some(&chunk5));
        assert!(cache.chunk(Coord2 { x: 5, y: 0 }).is_none());
        // last = 4, 2, 5, 1 = first
        assert!(cache.load(Coord2 { x: 5, y: 0 }, chunk1.clone()).is_none());

        // last = 4, 2, 5, 1 = first
        cache
            .entry_mut(pack_point(Coord2 { x: 5, y: 0 }, Coord2 { x: 0, y: 0 }))
            .unwrap()
            .ground = Ground::Sand;
        chunk1.entries[[0, 0]].ground = Ground::Sand;

        assert_eq!(cache.chunk(Coord2 { x: 5, y: 0 }), Some(&chunk1));
        assert!(cache.needs_flush.contains(&Coord2 { x: 5, y: 0 }));

        // last = 4, 2, 1, 5 = first
        cache
            .entry_mut(pack_point(Coord2 { x: 2, y: 0 }, Coord2 { x: 0, y: 1 }))
            .unwrap()
            .ground = Ground::Rock;
        chunk5.entries[[1, 0]].ground = Ground::Rock;

        // last = 4, 2, 1, 5 = first
        assert_eq!(cache.chunk(Coord2 { x: 2, y: 0 }), Some(&chunk5));
        assert!(cache.needs_flush.contains(&Coord2 { x: 2, y: 0 }));
        assert!(!cache.needs_flush.contains(&Coord2 { x: 1, y: 1 }));

        // last = 2, 1, 5, 4 = first
        cache.access(Coord2 { x: 1, y: 1 });

        // last = 1, 5, 4, 2 = first
        cache.access(Coord2 { x: 1, y: 0 });

        // last = 5, 4, 2, 3 = first
        assert_eq!(
            cache.load(Coord2 { x: 0, y: 1 }, chunk3.clone()),
            Some((Coord2 { x: 5, y: 0 }, chunk1.clone()))
        );
    }
}
