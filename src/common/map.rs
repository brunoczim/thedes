use crate::common::{biome::Biome, block::Block, ground::Ground, thede};
use gardiz::coord::Vec2;
use ndarray::{Array, Ix, Ix2};
use std::{
    future::Future,
    ops::{Index, IndexMut},
};

pub type Coord = u16;

pub const CHUNK_SIZE_EXP: Vec2<Coord> = Vec2 { x: 5, y: 5 };
pub const CHUNK_SIZE: Vec2<Coord> =
    Vec2 { x: 1 << CHUNK_SIZE_EXP.x, y: 1 << CHUNK_SIZE_EXP.y };
pub const CHUNK_SHAPE: [Ix; 2] = [CHUNK_SIZE.y as usize, CHUNK_SIZE.x as usize];

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Entry {
    pub biome: Biome,
    pub ground: Ground,
    pub block: Block,
    pub thede: Option<thede::Id>,
}

#[derive(Debug, Clone)]
pub struct ChunkBuilderStep {
    current: Vec2<Coord>,
    entries: Vec<Entry>,
}

impl ChunkBuilderStep {
    fn new() -> Self {
        Self { current: Vec2 { x: 0, y: 0 }, entries: Vec::new() }
    }

    pub fn offset(&self) -> Vec2<Coord> {
        self.current
    }

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
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ChunkBuilder {
    fn default() -> Self {
        ChunkBuilder::Step(ChunkBuilderStep::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Chunk {
    entries: Array<Entry, Ix2>,
}

impl Chunk {
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

    pub fn len(&self) -> Vec2<Coord> {
        let (y, x) = self.entries.dim();
        Vec2 { y: y as Coord, x: x as Coord }
    }

    pub fn get(&self, index: Vec2<Coord>) -> Option<&Entry> {
        self.entries.get([usize::from(index.y), usize::from(index.x)])
    }

    pub fn get_mut(&mut self, index: Vec2<Coord>) -> Option<&mut Entry> {
        self.entries.get_mut([usize::from(index.y), usize::from(index.x)])
    }
}

impl Index<Vec2<Coord>> for Chunk {
    type Output = Entry;

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

pub fn unpack_chunk(point: Vec2<Coord>) -> Vec2<Coord> {
    point.zip_with(CHUNK_SIZE_EXP, |coord, exp| coord >> exp)
}

pub fn unpack_offset(point: Vec2<Coord>) -> Vec2<Coord> {
    point.zip_with(CHUNK_SIZE_EXP, |coord, exp| coord & ((1 << exp) - 1))
}

pub fn pack_point(chunk: Vec2<Coord>, offset: Vec2<Coord>) -> Vec2<Coord> {
    chunk.zip(offset).zip_with(CHUNK_SIZE_EXP, |(chunk, offset), exp| {
        chunk << exp | offset & ((1 << exp) - 1)
    })
}
