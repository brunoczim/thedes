#[cfg(test)]
mod test;

use crate::{biome::Biome, block::Block, ground::Ground, thede};
use gardiz::coord::Vec2;
use ndarray::{Array, Ix, Ix2};
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    ops::{Index, IndexMut},
};

pub type Coord = u16;

pub const CHUNK_SIZE_EXP: Vec2<Coord> = Vec2 { x: 5, y: 5 };
pub const CHUNK_SIZE: Vec2<Coord> =
    Vec2 { x: 1 << CHUNK_SIZE_EXP.x, y: 1 << CHUNK_SIZE_EXP.y };
pub const CHUNK_SHAPE: [Ix; 2] = [CHUNK_SIZE.y as usize, CHUNK_SIZE.x as usize];

pub const MIN_CACHE_LIMIT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Entry {
    pub biome: Biome,
    pub ground: Ground,
    pub block: Block,
    pub thede: thede::MapData,
}

impl Default for Entry {
    #[inline]
    fn default() -> Self {
        Self {
            biome: Biome::Unknown,
            ground: Ground::Unknown,
            block: Block::Unknown,
            thede: thede::MapData::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkBuilderStep {
    current: Vec2<Coord>,
    entries: Vec<Entry>,
}

impl ChunkBuilderStep {
    #[inline]
    fn new() -> Self {
        Self { current: Vec2 { x: 0, y: 0 }, entries: Vec::new() }
    }

    #[inline]
    pub fn offset(&self) -> Vec2<Coord> {
        self.current
    }

    #[inline]
    pub fn next(mut self, entry: Entry) -> ChunkBuilder {
        self.entries.push(entry);
        self.current.x += 1;

        if self.current.x >= CHUNK_SIZE.x {
            self.current.x = 0;
            self.current.y += 1;
            if self.current.y >= CHUNK_SIZE.y {
                return ChunkBuilder::Done(Chunk {
                    entries: Array::from_shape_vec(CHUNK_SHAPE, self.entries)
                        .expect("chunk shape"),
                });
            }
        }

        ChunkBuilder::Step(self)
    }
}

#[derive(Debug, Clone)]
pub enum ChunkBuilder {
    Step(ChunkBuilderStep),
    Done(Chunk),
}

impl ChunkBuilder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ChunkBuilder {
    #[inline]
    fn default() -> Self {
        ChunkBuilder::Step(ChunkBuilderStep::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Chunk {
    entries: Array<Entry, Ix2>,
}

impl Default for Chunk {
    #[inline]
    fn default() -> Self {
        Self { entries: Array::default(CHUNK_SHAPE) }
    }
}

impl Chunk {
    #[inline]
    pub fn from_offsets<F>(mut mapper: F) -> Self
    where
        F: FnMut(Vec2<Coord>) -> Entry,
    {
        let mut builder = ChunkBuilder::default();
        loop {
            match builder {
                ChunkBuilder::Step(step) => {
                    let offset = step.offset();
                    builder = step.next(mapper(offset));
                },
                ChunkBuilder::Done(chunk) => break chunk,
            }
        }
    }

    #[inline]
    pub fn from_fallible_offsets<F, E>(mut mapper: F) -> Result<Self, E>
    where
        F: FnMut(Vec2<Coord>) -> Result<Entry, E>,
    {
        let mut builder = ChunkBuilder::default();
        loop {
            match builder {
                ChunkBuilder::Step(step) => {
                    let offset = step.offset();
                    builder = step.next(mapper(offset)?);
                },
                ChunkBuilder::Done(chunk) => break Ok(chunk),
            }
        }
    }

    #[inline]
    pub async fn from_future_offsets<F, A>(mut mapper: F) -> Self
    where
        F: FnMut(Vec2<Coord>) -> A,
        A: Future<Output = Entry>,
    {
        let mut builder = ChunkBuilder::default();
        loop {
            match builder {
                ChunkBuilder::Step(step) => {
                    let offset = step.offset();
                    builder = step.next(mapper(offset).await);
                },
                ChunkBuilder::Done(chunk) => break chunk,
            }
        }
    }

    #[inline]
    pub async fn from_fail_fut_offsets<F, A, E>(
        mut mapper: F,
    ) -> Result<Self, E>
    where
        F: FnMut(Vec2<Coord>) -> A,
        A: Future<Output = Result<Entry, E>>,
    {
        let mut builder = ChunkBuilder::default();
        loop {
            match builder {
                ChunkBuilder::Step(step) => {
                    let offset = step.offset();
                    builder = step.next(mapper(offset).await?);
                },

                ChunkBuilder::Done(chunk) => break Ok(chunk),
            }
        }
    }

    #[inline]
    pub fn len(&self) -> Vec2<Coord> {
        let (y, x) = self.entries.dim();
        Vec2 { y: y as Coord, x: x as Coord }
    }

    #[inline]
    pub fn get(&self, index: Vec2<Coord>) -> Option<&Entry> {
        self.entries.get([usize::from(index.y), usize::from(index.x)])
    }

    #[inline]
    pub fn get_mut(&mut self, index: Vec2<Coord>) -> Option<&mut Entry> {
        self.entries.get_mut([usize::from(index.y), usize::from(index.x)])
    }
}

impl Index<Vec2<Coord>> for Chunk {
    type Output = Entry;

    #[inline]
    fn index(&self, index: Vec2<Coord>) -> &Self::Output {
        #[cold]
        #[inline(never)]
        fn invalid_index(index: Vec2<Coord>, length: Vec2<Coord>) -> ! {
            panic!(
                "Invalid coordinate offset {} for chunk of length {}",
                index, length
            )
        }

        match self.get(index) {
            Some(entry) => entry,
            None => invalid_index(index, self.len()),
        }
    }
}

impl IndexMut<Vec2<Coord>> for Chunk {
    #[inline]
    fn index_mut(&mut self, index: Vec2<Coord>) -> &mut Self::Output {
        #[cold]
        #[inline(never)]
        fn invalid_index(index: Vec2<Coord>, length: Vec2<Coord>) -> ! {
            panic!(
                "Invalid coordinate offset {} for chunk of length {}",
                index, length
            )
        }

        let length = self.len();
        match self.get_mut(index) {
            Some(entry) => entry,
            None => invalid_index(index, length),
        }
    }
}

#[inline]
pub fn unpack_chunk(point: Vec2<Coord>) -> Vec2<Coord> {
    point.zip_with(CHUNK_SIZE_EXP, |coord, exp| coord >> exp)
}

#[inline]
pub fn unpack_offset(point: Vec2<Coord>) -> Vec2<Coord> {
    point.zip_with(CHUNK_SIZE_EXP, |coord, exp| coord & ((1 << exp) - 1))
}

#[inline]
pub fn pack_point(chunk: Vec2<Coord>, offset: Vec2<Coord>) -> Vec2<Coord> {
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
pub struct Cache {
    limit: usize,
    needs_flush: HashSet<Vec2<Coord>>,
    chunks: HashMap<Vec2<Coord>, CachedChunk>,
    first: Option<Vec2<Coord>>,
    last: Option<Vec2<Coord>>,
}

impl Cache {
    #[inline]
    pub fn new(limit: usize) -> Self {
        Self {
            limit: limit.max(MIN_CACHE_LIMIT),
            needs_flush: HashSet::new(),
            chunks: HashMap::with_capacity(limit),
            first: None,
            last: None,
        }
    }

    #[inline]
    pub fn chunk(&mut self, index: Vec2<Coord>) -> Option<&Chunk> {
        if self.chunks.contains_key(&index) {
            self.access(index);
        }
        self.chunks.get(&index).map(|cached| &cached.chunk)
    }

    #[inline]
    pub fn chunk_mut(&mut self, index: Vec2<Coord>) -> Option<&mut Chunk> {
        if self.chunks.contains_key(&index) {
            self.access(index);
            self.needs_flush.insert(index);
        }
        self.chunks.get_mut(&index).map(|cached| &mut cached.chunk)
    }

    #[inline]
    pub fn entry(&mut self, point: Vec2<Coord>) -> Option<&Entry> {
        let chunk_index = unpack_chunk(point);
        let offset = unpack_offset(point);
        self.chunk(chunk_index)
            .map(|chunk| &chunk.entries[[offset.y as usize, offset.x as usize]])
    }

    #[inline]
    pub fn entry_mut(&mut self, point: Vec2<Coord>) -> Option<&mut Entry> {
        let chunk_index = unpack_chunk(point);
        let offset = unpack_offset(point);
        self.chunk_mut(chunk_index).map(|chunk| {
            &mut chunk.entries[[offset.y as usize, offset.x as usize]]
        })
    }

    #[inline]
    pub fn load(
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
    #[inline]
    pub fn drop_oldest(&mut self) -> Option<(Vec2<Coord>, Chunk)> {
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

    #[inline]
    pub fn access(&mut self, chunk_index: Vec2<Coord>) {
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

    #[inline]
    pub fn debug_iter(&self) -> CacheDebugIter {
        CacheDebugIter {
            cache: self,
            front_back: self
                .first
                .and_then(|front| self.last.map(|back| (front, back))),
        }
    }
}

#[derive(Debug)]
pub struct CacheDebugIter<'cache> {
    cache: &'cache Cache,
    front_back: Option<(Vec2<Coord>, Vec2<Coord>)>,
}

impl<'cache> Iterator for CacheDebugIter<'cache> {
    type Item = Vec2<Coord>;

    #[inline]
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
    #[inline]
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
