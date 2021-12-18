use gardiz::coord::Vec2;
use kopidaz::tree::Tree;
use std::collections::HashSet;
use thedes_common::{
    error::Result,
    map::{unpack_chunk, unpack_offset, Cache, Chunk, Coord, Entry},
    ResultExt,
};

#[derive(Debug, Clone)]
pub struct Navigator {
    map: Map,
    /*
    biome_gen: biome::Generator,
    block_gen: block::Generator,
    thede_gen: thede::Generator,
    */
}

impl Navigator {
    pub fn new(map: Map) -> Self {
        Self { map }
    }

    pub fn map(&mut self) -> &mut Map {
        &mut self.map
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    cache: Cache,
    tree: Tree<Vec2<Coord>, Chunk>,
    fresh_chunks: HashSet<Vec2<Coord>>,
}

impl Map {
    pub async fn new(db: &sled::Db, cache_limit: usize) -> Result<Self> {
        Ok(Self {
            cache: Cache::new(cache_limit),
            tree: Tree::open(db, "Map").await.erase_err()?,
            fresh_chunks: HashSet::new(),
        })
    }

    pub async fn chunk(&mut self, chunk_index: Vec2<Coord>) -> Result<&Chunk> {
        if self.cache.chunk(chunk_index).is_none() {
            let chunk = match self.tree.get(&chunk_index).await.erase_err()? {
                Some(chunk) => chunk,
                None => {
                    self.fresh_chunks.insert(chunk_index);
                    Chunk::default()
                },
            };
            self.cache.load(chunk_index, chunk);
        }
        Ok(self.cache.chunk(chunk_index).unwrap())
    }

    pub async fn chunk_mut(
        &mut self,
        chunk_index: Vec2<Coord>,
    ) -> Result<&mut Chunk> {
        if self.cache.chunk_mut(chunk_index).is_none() {
            let chunk = match self.tree.get(&chunk_index).await.erase_err()? {
                Some(chunk) => chunk,
                None => {
                    self.fresh_chunks.insert(chunk_index);
                    Chunk::default()
                },
            };
            self.cache.load(chunk_index, chunk);
        }
        Ok(self.cache.chunk_mut(chunk_index).unwrap())
    }

    pub async fn entry(&mut self, point: Vec2<Coord>) -> Result<&Entry> {
        let chunk = self.chunk(unpack_chunk(point)).await?;
        Ok(&chunk[unpack_offset(point)])
    }

    pub async fn entry_mut(
        &mut self,
        point: Vec2<Coord>,
    ) -> Result<&mut Entry> {
        let chunk = self.chunk_mut(unpack_chunk(point)).await?;
        Ok(&mut chunk[unpack_offset(point)])
    }
}
