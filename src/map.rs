use crate::{
    entity::{biome, npc, thede, Biome},
    error::Result,
    math::{
        plane::{Coord2, Nat, Rect},
        rand::Seed,
    },
    matter::{Block, Ground},
    storage::save::Tree,
};
use ndarray::{Array, Ix, Ix2};
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    sync::Arc,
};

const CHUNK_SIZE_EXP: Coord2<Nat> = Coord2 { x: 6, y: 6 };
const CHUNK_SIZE: Coord2<Nat> =
    Coord2 { x: 1 << CHUNK_SIZE_EXP.x, y: 1 << CHUNK_SIZE_EXP.y };
const CHUNK_SHAPE: [Ix; 2] = [CHUNK_SIZE.y as usize, CHUNK_SIZE.x as usize];
const MIN_CACHE_LIMIT: usize = 4;

pub const RECOMMENDED_CACHE_LIMIT: usize = 64;

#[derive(Debug, Clone)]
pub struct Map {
    cache: Cache,
    tree: Tree<Coord2<Nat>, Chunk>,
    biome_gen: Arc<biome::Generator>,
    thede_gen: Arc<thede::Generator>,
}

impl Map {
    pub async fn new(
        db: &sled::Db,
        seed: Seed,
        cache_limit: usize,
    ) -> Result<Self> {
        let cache = Cache::new(cache_limit);
        let tree = Tree::open(db, "Map").await?;
        let biome_gen = Arc::new(biome::Generator::new(seed));
        let thede_gen = Arc::new(thede::Generator::new(seed));
        Ok(Self { cache, tree, biome_gen, thede_gen })
    }

    pub async fn flush(&mut self) -> Result<()> {
        for &index in self.cache.needs_flush.iter() {
            self.tree.insert(&index, &self.cache.chunks[&index].chunk).await?;
        }
        self.cache.needs_flush.clear();
        Ok(())
    }

    pub async fn entry(
        &mut self,
        point: Coord2<Nat>,
        db: &sled::Db,
        thedes: &thede::Registry,
        npcs: &npc::Registry,
        seed: Seed,
    ) -> Result<&Entry> {
        self.require_chunk(unpack_chunk(point), db, thedes, npcs, seed).await?;
        Ok(self.cache.entry(point).expect("I just loaded it"))
    }

    pub async fn entry_mut(
        &mut self,
        point: Coord2<Nat>,
        db: &sled::Db,
        thedes: &thede::Registry,
        npcs: &npc::Registry,
        seed: Seed,
    ) -> Result<&mut Entry> {
        self.require_chunk(unpack_chunk(point), db, thedes, npcs, seed).await?;
        Ok(self.cache.entry_mut(point).expect("I just loaded it"))
    }

    async fn require_chunk(
        &mut self,
        index: Coord2<Nat>,
        db: &sled::Db,
        thedes: &thede::Registry,
        npcs: &npc::Registry,
        seed: Seed,
    ) -> Result<()> {
        GeneratingMap::new(self)
            .generate(index, db, thedes, npcs, seed)
            .await?;
        self.load_chunk(index).await?;
        Ok(())
    }

    async fn load_chunk(&mut self, index: Coord2<Nat>) -> Result<bool> {
        if self.cache.chunk(index).is_some() {
            Ok(true)
        } else if let Some(chunk) = self.tree.get(&index).await? {
            self.cache.load(index, chunk);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn init_chunk(&mut self, index: Coord2<Nat>) -> Result<()> {
        let chunk = Chunk::new(|offset| {
            let biome = self.biome_gen.biome_at(pack_point(index, offset));
            Entry {
                biome,
                ground: biome.main_ground(),
                block: Block::Empty,
                thede: None,
            }
        });

        self.tree.insert(&index, &chunk).await?;
        self.cache.load(index, chunk);
        Ok(())
    }
}

#[derive(Debug)]
pub struct GeneratingMap<'map> {
    map: &'map mut Map,
    thede_requests: Vec<Coord2<Nat>>,
}

impl<'map> GeneratingMap<'map> {
    fn new(map: &'map mut Map) -> Self {
        Self { map, thede_requests: Vec::new() }
    }

    async fn generate(
        &mut self,
        index: Coord2<Nat>,
        db: &sled::Db,
        thedes: &thede::Registry,
        npcs: &npc::Registry,
        seed: Seed,
    ) -> Result<()> {
        self.thede_requests.push(index);
        while let Some(requested_chunk) = self.thede_requests.pop() {
            self.gen_thedes(requested_chunk, db, thedes, npcs, seed).await?;
        }
        Ok(())
    }

    async fn gen_thedes(
        &mut self,
        index: Coord2<Nat>,
        db: &sled::Db,
        thedes: &thede::Registry,
        npcs: &npc::Registry,
        seed: Seed,
    ) -> Result<()> {
        let rect = Rect {
            start: pack_point(index, Coord2::from_axes(|_| 0)),
            size: CHUNK_SIZE,
        };

        // otherwise borrow checker gets sad
        let thede_gen = self.map.thede_gen.clone();

        for point in rect.lines() {
            let is_thede = thede_gen.is_thede_at(point);
            let is_none =
                self.map.cache.entry(point).expect("bad point").thede.is_none();
            if is_thede && is_none {
                thede_gen.generate(point, db, thedes, npcs, self, seed).await?;
            }
        }

        Ok(())
    }

    pub(crate) async fn entry(&mut self, point: Coord2<Nat>) -> Result<&Entry> {
        self.require_chunk(unpack_chunk(point)).await?;
        Ok(self.map.cache.entry(point).expect("Just loaded it"))
    }

    pub(crate) async fn entry_mut(
        &mut self,
        point: Coord2<Nat>,
    ) -> Result<&mut Entry> {
        self.require_chunk(unpack_chunk(point)).await?;
        Ok(self.map.cache.entry_mut(point).expect("Just loaded it"))
    }

    async fn require_chunk(&mut self, index: Coord2<Nat>) -> Result<()> {
        if !self.map.load_chunk(index).await? {
            self.map.init_chunk(index).await?;
            self.thede_requests.push(index);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Entry {
    pub biome: Biome,
    pub ground: Ground,
    pub block: Block,
    pub thede: Option<thede::Id>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Chunk {
    entries: Array<Entry, Ix2>,
}

impl Chunk {
    fn new<F>(mut entry_maker: F) -> Self
    where
        F: FnMut(Coord2<Nat>) -> Entry,
    {
        Self {
            entries: Array::from_shape_fn(CHUNK_SHAPE, |(y, x)| {
                entry_maker(Coord2 { y: y as Nat, x: x as Nat })
            }),
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            entries: Array::from_elem(
                CHUNK_SHAPE,
                Entry {
                    biome: Biome::Plain,
                    ground: Ground::Grass,
                    block: Block::Empty,
                    thede: None,
                },
            ),
        }
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
            chunks: HashMap::new(),
            first: None,
            last: None,
        }
    }

    fn chunk(&mut self, point: Coord2<Nat>) -> Option<&Chunk> {
        let chunk_index = unpack_chunk(point);
        if self.chunks.contains_key(&chunk_index) {
            self.access(chunk_index);
        }
        self.chunks.get(&chunk_index).map(|cached| &cached.chunk)
    }

    fn chunk_mut(&mut self, point: Coord2<Nat>) -> Option<&mut Chunk> {
        let chunk_index = unpack_chunk(point);
        if self.chunks.contains_key(&chunk_index) {
            self.access(chunk_index);
            self.needs_flush.insert(chunk_index);
        }
        self.chunks.get_mut(&chunk_index).map(|cached| &mut cached.chunk)
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

    fn load(&mut self, chunk_index: Coord2<Nat>, chunk: Chunk) {
        if self.chunks.len() >= self.limit {
            self.drop_oldest();
        }

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
    }

    fn drop_oldest(&mut self) {
        if let Some(last) = self.last {
            let last_prev = self.chunks.get_mut(&last).expect("bad list").prev;
            if let Some(prev) = last_prev {
                self.chunks.get_mut(&prev).expect("bad list").next = None;
            }
            if self.first == self.last {
                self.first = last_prev;
            }
            self.last = last_prev;
        }
    }

    fn access(&mut self, chunk_index: Coord2<Nat>) {
        let (chunk_prev, chunk_next) = {
            let chunk = &self.chunks[&chunk_index];
            (chunk.prev, chunk.next)
        };

        if let Some(prev) = chunk_prev {
            self.chunks.get_mut(&prev).expect("bad list").next = chunk_next;
        }
        if let Some(next) = chunk_next {
            self.chunks.get_mut(&next).expect("bad list").prev = chunk_prev;
        }

        if let Some(first) = self.first {
            let refer = self.chunks.get_mut(&first).expect("bad list");
            refer.prev = Some(chunk_index);
        }
        {
            let chunk = self.chunks.get_mut(&chunk_index).expect("bad list");
            chunk.prev = None;
            chunk.next = self.first;
        }
        self.first = Some(chunk_index);
        if self.last.is_none() {
            self.last = self.first;
        }
    }
}
