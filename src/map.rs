use crate::{
    entity::{biome, thede, Biome},
    error::Result,
    math::rand::Seed,
    matter::{block, Block, Ground},
    storage::save::SavedGame,
};
use gardiz::coord::Vec2;
use kopidaz::tree::Tree;
use ndarray::{Array, Ix, Ix2};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{Mutex, MutexGuard};

pub type Coord = u16;

const CHUNK_SIZE_EXP: Vec2<Coord> = Vec2 { x: 5, y: 5 };
const CHUNK_SIZE: Vec2<Coord> =
    Vec2 { x: 1 << CHUNK_SIZE_EXP.x, y: 1 << CHUNK_SIZE_EXP.y };
const CHUNK_SHAPE: [Ix; 2] = [CHUNK_SIZE.y as usize, CHUNK_SIZE.x as usize];
const MIN_CACHE_LIMIT: usize = 4;

/// The recommended cache limit of the map.
pub const RECOMMENDED_CACHE_LIMIT: usize = 128;

fn layer_must_be_set() -> ! {
    panic!("Map layer required to be set, but it isn't")
}

fn layer_must_not_be_gening() -> ! {
    panic!("Map layer required to not being generating, but it is")
}

/// A raw entry of a map layer.
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
    /// The entry of this layer is set to some data.
    Set(T),
    /// The entry is being generated.
    Generating,
    /// The entry is not generated yet.
    NotGenerated,
}

impl<T> RawLayer<T> {
    /// Converts the data into a reference, if set.
    pub fn as_ref(&self) -> RawLayer<&T> {
        match self {
            RawLayer::Set(val) => RawLayer::Set(val),
            RawLayer::Generating => RawLayer::Generating,
            RawLayer::NotGenerated => RawLayer::NotGenerated,
        }
    }

    /// Converts the data into a mutable reference, if set.
    pub fn as_mut(&mut self) -> RawLayer<&mut T> {
        match self {
            RawLayer::Set(val) => RawLayer::Set(val),
            RawLayer::Generating => RawLayer::Generating,
            RawLayer::NotGenerated => RawLayer::NotGenerated,
        }
    }

    /// If the entry is not set to some data, panics. Returns the data.
    ///
    /// # Panics
    /// Panics if not set.
    pub fn must_be_set(self) -> T {
        match self {
            RawLayer::Set(val) => val,
            _ => layer_must_be_set(),
        }
    }

    /// If the entry is generating, panics. Returns the data if set.
    ///
    /// # Panics
    /// Panics if the entry is generating.
    pub fn must_not_be_gening(self) -> Option<T> {
        match self {
            RawLayer::Set(val) => Some(val),
            RawLayer::NotGenerated => None,
            RawLayer::Generating => layer_must_not_be_gening(),
        }
    }

    /// Merges the case of `Generating` and `NotGenerating` into a `None`, while
    /// wrapping the data if set into a `Some`.
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

/// A centralized game map.
#[derive(Debug, Clone)]
pub struct Map {
    inner: Arc<Mutex<MapInner>>,
}

impl Map {
    /// Initializes this map. Recommended to use `RECOMMENDED_CACHE_LIMIT` as
    /// `cache_limit`.
    pub async fn new(
        db: &sled::Db,
        seed: Seed,
        cache_limit: usize,
    ) -> Result<Self> {
        let inner = MapInner {
            cache: Cache::new(cache_limit),
            tree: Tree::open(db, "Map").await?,
            biome_gen: Arc::new(biome::Generator::new(seed)),
            block_gen: Arc::new(block::Generator::new(seed)),
            thede_gen: Arc::new(thede::Generator::new(seed)),
        };
        Ok(Self { inner: Arc::new(Mutex::new(inner)) })
    }

    /// Flushes the cache. **MUST BE CALLED** before drop.
    pub async fn flush(&self) -> Result<()> {
        let mut locked = self.locked();
        let inner = locked.inner().await;
        for coord in &inner.cache.needs_flush {
            inner.tree.insert(coord, &inner.cache.chunks[coord].chunk).await?;
        }
        inner.cache.needs_flush.clear();
        Ok(())
    }

    /// Returns the biome layer's raw entry for a given point. Does not
    /// auto-generate it.
    pub async fn biome_raw(
        &self,
        point: Vec2<Coord>,
    ) -> Result<RawLayer<Biome>> {
        let ret = self.locked().entry(point).await?.biome.clone();
        Ok(ret)
    }

    /// Sets the biome layer's raw entry for a given point. Does not
    /// auto-generate it.
    pub async fn set_biome_raw(
        &self,
        point: Vec2<Coord>,
        biome: Biome,
    ) -> Result<()> {
        self.locked().entry(point).await?.biome = RawLayer::Set(biome);
        Ok(())
    }

    /// Returns the biome layer's entry for a given point. Auto generates if it
    /// is not generated.
    pub async fn biome(&self, point: Vec2<Coord>) -> Result<Biome> {
        let ret = self.locked().biome(point).await?.clone();
        Ok(ret)
    }

    /// Sets the biome layer's entry for a given point. Before setting, auto
    /// generates if it is not generated.
    pub async fn set_biome(
        &self,
        point: Vec2<Coord>,
        biome: Biome,
    ) -> Result<()> {
        *self.locked().biome(point).await? = biome;
        Ok(())
    }

    /// Returns the ground layer's raw entry for a given point. Does not
    /// auto-generate it.
    pub async fn ground_raw(
        &self,
        point: Vec2<Coord>,
    ) -> Result<RawLayer<Ground>> {
        let ret = self.locked().entry(point).await?.ground.clone();
        Ok(ret)
    }

    /// Sets the ground layer's raw entry for a given point. Does not
    /// auto-generate it.
    pub async fn set_ground_raw(
        &self,
        point: Vec2<Coord>,
        ground: Ground,
    ) -> Result<()> {
        self.locked().entry(point).await?.ground = RawLayer::Set(ground);
        Ok(())
    }

    /// Returns the ground layer's entry for a given point. Auto generates if it
    /// is not generated.
    pub async fn ground(&self, point: Vec2<Coord>) -> Result<Ground> {
        let ret = self.locked().ground(point).await?.clone();
        Ok(ret)
    }

    /// Sets the ground layer's entry for a given point. Before setting, auto
    /// generates if it is not generated.
    pub async fn set_ground(
        &self,
        point: Vec2<Coord>,
        ground: Ground,
    ) -> Result<()> {
        *self.locked().ground(point).await? = ground;
        Ok(())
    }

    /// Returns the block layer's raw entry for a given point. Does not
    /// auto-generate it.
    pub async fn block_raw(
        &self,
        point: Vec2<Coord>,
    ) -> Result<RawLayer<Block>> {
        let ret = self.locked().entry(point).await?.block.clone();
        Ok(ret)
    }

    /// Sets the block layer's raw entry for a given point. Does not
    /// auto-generate it.
    pub async fn set_block_raw(
        &self,
        point: Vec2<Coord>,
        block: Block,
    ) -> Result<()> {
        self.locked().entry(point).await?.block = RawLayer::Set(block);
        Ok(())
    }

    /// Returns the block layer's entry for a given point. Auto generates if it
    /// is not generated.
    pub async fn block(&self, point: Vec2<Coord>) -> Result<Block> {
        let ret = self.locked().block(point).await?.clone();
        Ok(ret)
    }

    /// Sets the block layer's entry for a given point. Before setting, auto
    /// generates if it is not generated.
    pub async fn set_block(
        &self,
        point: Vec2<Coord>,
        block: Block,
    ) -> Result<()> {
        *self.locked().block(point).await? = block;
        Ok(())
    }

    /// Returns the thede layer's raw entry for a given point. Does not
    /// auto-generate it.
    pub async fn thede_raw(
        &self,
        point: Vec2<Coord>,
    ) -> Result<RawLayer<thede::MapLayer>> {
        let ret = self.locked().entry(point).await?.thede.clone();
        Ok(ret)
    }

    /// Sets the thede layer's raw entry for a given point. Does not
    /// auto-generate it.
    pub async fn set_thede_raw(
        &self,
        point: Vec2<Coord>,
        thede: thede::MapLayer,
    ) -> Result<()> {
        self.locked().entry(point).await?.thede = RawLayer::Set(thede);
        Ok(())
    }

    /// Returns the thede layer's entry for a given point. Auto generates if it
    /// is not generated.
    ///
    /// # Panics
    /// Panics if called while already generating this point.
    pub async fn thede(
        &self,
        point: Vec2<Coord>,
        game: &SavedGame,
    ) -> Result<thede::MapLayer> {
        let ret = self.locked().thede(point, game).await?.clone();
        Ok(ret)
    }

    /// Sets the thede layer's entry for a given point. Before setting, auto
    /// generates if it is not generated.
    ///
    /// # Panics
    /// Panics if called while already generating this point.
    pub async fn set_thede(
        &self,
        point: Vec2<Coord>,
        thede: thede::MapLayer,
        game: &SavedGame,
    ) -> Result<()> {
        *self.locked().thede(point, game).await? = thede;
        Ok(())
    }

    fn locked<'map>(&'map self) -> LockedMap<'map> {
        LockedMap { guard: None, map: self }
    }
}

#[derive(Debug, Clone)]
struct MapInner {
    cache: Cache,
    tree: Tree<Vec2<Coord>, Chunk>,
    biome_gen: Arc<biome::Generator>,
    block_gen: Arc<block::Generator>,
    thede_gen: Arc<thede::Generator>,
}

#[derive(Debug)]
struct LockedMap<'map> {
    guard: Option<MutexGuard<'map, MapInner>>,
    map: &'map Map,
}

impl<'map> LockedMap<'map> {
    async fn entry(&mut self, point: Vec2<Coord>) -> Result<&mut Entry> {
        let index = unpack_chunk(point);
        self.require_chunk(index).await?;
        Ok(self.inner().await.cache.entry_mut(point).expect("I just loaded it"))
    }

    async fn biome(&mut self, point: Vec2<Coord>) -> Result<&mut Biome> {
        let needs_gen = self
            .entry(point)
            .await?
            .biome
            .as_mut()
            .must_not_be_gening()
            .is_none();

        if needs_gen {
            let biome = self.inner().await.biome_gen.biome_at(point);
            self.entry(point).await?.biome = RawLayer::Set(biome);
        }

        let biome = self.entry(point).await?.biome.as_mut().must_be_set();
        Ok(biome)
    }

    async fn ground(&mut self, point: Vec2<Coord>) -> Result<&mut Ground> {
        let needs_gen = self
            .entry(point)
            .await?
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

    async fn block(&mut self, point: Vec2<Coord>) -> Result<&mut Block> {
        let needs_gen = self
            .entry(point)
            .await?
            .block
            .as_mut()
            .must_not_be_gening()
            .is_none();

        if needs_gen {
            let block = self.inner().await.block_gen.block_at(point);
            self.entry(point).await?.block = RawLayer::Set(block);
        }

        let block = self.entry(point).await?.block.as_mut().must_be_set();
        Ok(block)
    }

    async fn thede(
        &mut self,
        point: Vec2<Coord>,
        game: &SavedGame,
    ) -> Result<&mut thede::MapLayer> {
        let needs_gen = self
            .entry(point)
            .await?
            .thede
            .as_mut()
            .must_not_be_gening()
            .is_none();

        if needs_gen {
            self.entry(point).await?.thede = RawLayer::Generating;
            let thede_gen = self.inner().await.thede_gen.clone();
            self.guard = None;
            thede_gen.generate(point, game).await?;
        }

        let thede = self.entry(point).await?.thede.as_mut().must_be_set();
        Ok(thede)
    }

    async fn inner(&mut self) -> &mut MapInner {
        if self.guard.is_none() {
            self.guard = Some(self.map.inner.lock().await);
        }

        &mut *self.guard.as_mut().expect("I checked it")
    }

    async fn load_chunk(&mut self, index: Vec2<Coord>) -> Result<bool> {
        if self.inner().await.cache.chunk(index).is_some() {
            Ok(true)
        } else if let Some(chunk) = self.inner().await.tree.get(&index).await? {
            if let Some((index, chunk)) =
                self.inner().await.cache.load(index, chunk)
            {
                self.inner().await.tree.insert(&index, &chunk).await?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn require_chunk(&mut self, index: Vec2<Coord>) -> Result<()> {
        if !self.load_chunk(index).await? {
            self.init_chunk(index).await?;
        }

        Ok(())
    }

    async fn init_chunk(&mut self, index: Vec2<Coord>) -> Result<()> {
        let chunk = Chunk::default();
        self.inner().await.tree.insert(&index, &chunk).await?;
        if let Some((index, chunk)) =
            self.inner().await.cache.load(index, chunk)
        {
            self.inner().await.tree.insert(&index, &chunk).await?;
        }

        Ok(())
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

fn unpack_chunk(point: Vec2<Coord>) -> Vec2<Coord> {
    point.zip_with(CHUNK_SIZE_EXP, |coord, exp| coord >> exp)
}

fn unpack_offset(point: Vec2<Coord>) -> Vec2<Coord> {
    point.zip_with(CHUNK_SIZE_EXP, |coord, exp| coord & ((1 << exp) - 1))
}

#[allow(dead_code)]
fn pack_point(chunk: Vec2<Coord>, offset: Vec2<Coord>) -> Vec2<Coord> {
    chunk.zip(offset).zip_with(CHUNK_SIZE_EXP, |(chunk, offset), exp| {
        chunk << exp | offset & ((1 << exp) - 1)
    })
}

#[derive(Debug, Clone)]
struct CachedChunk {
    chunk: Chunk,
    next: Option<Vec2<Coord>>,
    prev: Option<Vec2<Coord>>,
}

#[derive(Debug, Clone)]
struct Cache {
    limit: usize,
    needs_flush: HashSet<Vec2<Coord>>,
    chunks: HashMap<Vec2<Coord>, CachedChunk>,
    first: Option<Vec2<Coord>>,
    last: Option<Vec2<Coord>>,
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

    fn chunk(&mut self, index: Vec2<Coord>) -> Option<&Chunk> {
        if self.chunks.contains_key(&index) {
            self.access(index);
        }
        self.chunks.get(&index).map(|cached| &cached.chunk)
    }

    fn chunk_mut(&mut self, index: Vec2<Coord>) -> Option<&mut Chunk> {
        if self.chunks.contains_key(&index) {
            self.access(index);
            self.needs_flush.insert(index);
        }
        self.chunks.get_mut(&index).map(|cached| &mut cached.chunk)
    }

    #[allow(dead_code)]
    fn entry(&mut self, point: Vec2<Coord>) -> Option<&Entry> {
        let chunk_index = unpack_chunk(point);
        let offset = unpack_offset(point);
        self.chunk(chunk_index)
            .map(|chunk| &chunk.entries[[offset.y as usize, offset.x as usize]])
    }

    fn entry_mut(&mut self, point: Vec2<Coord>) -> Option<&mut Entry> {
        let chunk_index = unpack_chunk(point);
        let offset = unpack_offset(point);
        self.chunk_mut(chunk_index).map(|chunk| {
            &mut chunk.entries[[offset.y as usize, offset.x as usize]]
        })
    }

    #[must_use]
    fn load(
        &mut self,
        chunk_index: Vec2<Coord>,
        chunk: Chunk,
    ) -> Option<(Vec2<Coord>, Chunk)> {
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
    fn drop_oldest(&mut self) -> Option<(Vec2<Coord>, Chunk)> {
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

    fn access(&mut self, chunk_index: Vec2<Coord>) {
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
    front_back: Option<(Vec2<Coord>, Vec2<Coord>)>,
}

impl<'cache> Iterator for CacheDebugIter<'cache> {
    type Item = Vec2<Coord>;

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
    use super::{
        pack_point,
        unpack_chunk,
        unpack_offset,
        Cache,
        Chunk,
        RawLayer,
    };
    use crate::{entity::Biome, matter::Ground};
    use gardiz::coord::Vec2;

    #[test]
    fn pack_unpack() {
        let point1 = Vec2 { x: 4857, y: 7375 };
        assert_eq!(
            point1,
            pack_point(unpack_chunk(point1), unpack_offset(point1))
        );
    }

    #[test]
    fn cache() {
        let mut cache = Cache::new(4);
        let mut chunk1 = Chunk::default();
        chunk1.entries[[0, 0]].biome = RawLayer::Set(Biome::Desert);
        let mut chunk2 = Chunk::default();
        chunk2.entries[[0, 1]].biome = RawLayer::Set(Biome::Desert);
        let mut chunk3 = Chunk::default();
        chunk3.entries[[1, 1]].biome = RawLayer::Set(Biome::Desert);
        let mut chunk4 = Chunk::default();
        chunk4.entries[[0, 0]].biome = RawLayer::Set(Biome::RockDesert);
        let mut chunk5 = Chunk::default();
        chunk5.entries[[1, 0]].biome = RawLayer::Set(Biome::RockDesert);
        // last = 1, 2, 3, 4 = first
        assert!(cache.load(Vec2 { x: 5, y: 0 }, chunk1.clone()).is_none());
        assert!(cache.load(Vec2 { x: 1, y: 0 }, chunk2.clone()).is_none());
        assert!(cache.load(Vec2 { x: 0, y: 1 }, chunk3.clone()).is_none());
        assert!(cache.load(Vec2 { x: 1, y: 1 }, chunk4.clone()).is_none());

        // last = 2, 3, 4, 5 = first
        assert!(cache.load(Vec2 { x: 2, y: 0 }, chunk5.clone()).is_none());
        // last = 3, 4, 5, 2 = first
        assert_eq!(cache.chunk(Vec2 { x: 1, y: 0 }), Some(&chunk2));
        // last = 3, 4, 2, 5 = first
        assert_eq!(cache.chunk(Vec2 { x: 2, y: 0 }), Some(&chunk5));
        assert!(cache.chunk(Vec2 { x: 5, y: 0 }).is_none());
        // last = 4, 2, 5, 1 = first
        assert!(cache.load(Vec2 { x: 5, y: 0 }, chunk1.clone()).is_none());

        // last = 4, 2, 5, 1 = first
        cache
            .entry_mut(pack_point(Vec2 { x: 5, y: 0 }, Vec2 { x: 0, y: 0 }))
            .unwrap()
            .ground = RawLayer::Set(Ground::Sand);
        chunk1.entries[[0, 0]].ground = RawLayer::Set(Ground::Sand);

        assert_eq!(cache.chunk(Vec2 { x: 5, y: 0 }), Some(&chunk1));
        assert!(cache.needs_flush.contains(&Vec2 { x: 5, y: 0 }));

        // last = 4, 2, 1, 5 = first
        cache
            .entry_mut(pack_point(Vec2 { x: 2, y: 0 }, Vec2 { x: 0, y: 1 }))
            .unwrap()
            .ground = RawLayer::Set(Ground::Rock);
        chunk5.entries[[1, 0]].ground = RawLayer::Set(Ground::Rock);

        // last = 4, 2, 1, 5 = first
        assert_eq!(cache.chunk(Vec2 { x: 2, y: 0 }), Some(&chunk5));
        assert!(cache.needs_flush.contains(&Vec2 { x: 2, y: 0 }));
        assert!(!cache.needs_flush.contains(&Vec2 { x: 1, y: 1 }));

        // last = 2, 1, 5, 4 = first
        cache.access(Vec2 { x: 1, y: 1 });

        // last = 1, 5, 4, 2 = first
        cache.access(Vec2 { x: 1, y: 0 });

        // last = 5, 4, 2, 3 = first
        assert_eq!(
            cache.load(Vec2 { x: 0, y: 1 }, chunk3.clone()),
            Some((Vec2 { x: 5, y: 0 }, chunk1.clone()))
        );
    }
}
