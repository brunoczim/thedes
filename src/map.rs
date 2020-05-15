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
    sync::Arc,
};

const CHUNK_SIZE_EXP: Coord2<Nat> = Coord2 { x: 6, y: 6 };

const CHUNK_SIZE: Coord2<Nat> =
    Coord2 { x: 1 << CHUNK_SIZE_EXP.x, y: 1 << CHUNK_SIZE_EXP.y };

const CHUNK_SHAPE: [Ix; 2] = [CHUNK_SIZE.y as usize, CHUNK_SIZE.x as usize];

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
    fn new<F>(index: Coord2<Nat>, mut entry_maker: F) -> Self
    where
        F: FnMut(Coord2<Nat>) -> Entry,
    {
        Self {
            entries: Array::from_shape_fn(CHUNK_SHAPE, |(y, x)| {
                entry_maker(pack_point(
                    index,
                    Coord2 { y: y as Nat, x: x as Nat },
                ))
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
            limit: limit.max(1),
            needs_flush: HashSet::new(),
            chunks: HashMap::new(),
            first: None,
            last: None,
        }
    }

    fn entry(&mut self, point: Coord2<Nat>) -> Option<&Entry> {
        let chunk_index = unpack_chunk(point);
        let offset = unpack_offset(point);
        if self.chunks.contains_key(&chunk_index) {
            self.access(chunk_index);
        }
        self.chunks.get(&chunk_index).map(|cached| {
            &cached.chunk.entries[[offset.y as usize, offset.x as usize]]
        })
    }

    fn entry_mut(&mut self, point: Coord2<Nat>) -> Option<&mut Entry> {
        let chunk_index = unpack_chunk(point);
        let offset = unpack_offset(point);
        if self.chunks.contains_key(&chunk_index) {
            self.access(chunk_index);
            self.needs_flush.insert(chunk_index);
        }
        self.chunks.get_mut(&chunk_index).map(|cached| {
            &mut cached.chunk.entries[[offset.y as usize, offset.x as usize]]
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

#[derive(Debug, Clone)]
pub struct Map {
    cache: Cache,
    tree: Tree<Coord2<Nat>, Chunk>,
    biome_gen: biome::Generator,
    thede_gen: thede::Generator,
}

impl Map {
    pub async fn new(
        db: &sled::Db,
        seed: Seed,
        cache_limit: usize,
    ) -> Result<Self> {
        let cache = Cache::new(cache_limit);
        let tree = Tree::open(db, "Map").await?;
        let biome_gen = biome::Generator::new(seed);
        let thede_gen = thede::Generator::new(seed);
        Ok(Self { cache, tree, biome_gen, thede_gen })
    }
}
