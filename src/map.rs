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
    sync::Arc,
};

const CHUNK_SIZE_EXP: Coord2<Nat> = Coord2 { x: 1, y: 1 };
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
        game: &SavedGame,
    ) -> Result<&Entry> {
        self.require_chunk(unpack_chunk(point), game).await?;
        Ok(self.cache.entry(point).expect("I just loaded it"))
    }

    pub async fn entry_mut(
        &mut self,
        point: Coord2<Nat>,
        game: &SavedGame,
    ) -> Result<&mut Entry> {
        self.require_chunk(unpack_chunk(point), game).await?;
        Ok(self.cache.entry_mut(point).expect("I just loaded it"))
    }

    async fn require_chunk(
        &mut self,
        index: Coord2<Nat>,
        game: &SavedGame,
    ) -> Result<()> {
        if !self.load_chunk(index).await? {
            GeneratingMap::new(self).generate(index, game).await?;
            self.load_chunk(index).await?;
        }
        Ok(())
    }

    async fn load_chunk(&mut self, index: Coord2<Nat>) -> Result<bool> {
        if self.cache.chunk(index).is_some() {
            Ok(true)
        } else if let Some(chunk) = self.tree.get(&index).await? {
            if let Some((index, chunk)) = self.cache.load(index, chunk) {
                self.tree.insert(&index, &chunk).await?;
            }
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
                thede_visited: false,
            }
        });

        self.tree.insert(&index, &chunk).await?;
        if let Some((index, chunk)) = self.cache.load(index, chunk) {
            self.tree.insert(&index, &chunk).await?;
        }
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
        game: &SavedGame,
    ) -> Result<()> {
        self.map.init_chunk(index).await?;
        self.thede_requests.push(index);
        while let Some(requested_chunk) = self.thede_requests.pop() {
            self.gen_thedes(requested_chunk, game).await?;
        }
        Ok(())
    }

    async fn gen_thedes(
        &mut self,
        index: Coord2<Nat>,
        game: &SavedGame,
    ) -> Result<()> {
        let rect = Rect {
            start: pack_point(index, Coord2::from_axes(|_| 0)),
            size: CHUNK_SIZE,
        };

        // otherwise borrow checker gets sad
        let thede_gen = self.map.thede_gen.clone();

        for point in rect.lines() {
            let is_thede = thede_gen.is_thede_at(point);
            self.map.load_chunk(index).await?;
            let is_empty = {
                let entry =
                    self.map.cache.entry_mut(point).expect("I just loaded it");
                let visited = entry.thede_visited;
                entry.thede_visited = true;
                !visited
            };
            if is_thede && is_empty {
                thede_gen.generate(point, game, self).await?;
            }
        }

        Ok(())
    }

    pub async fn entry(&mut self, point: Coord2<Nat>) -> Result<&Entry> {
        self.require_chunk(unpack_chunk(point)).await?;
        Ok(self.map.cache.entry(point).expect("Just loaded it"))
    }

    pub async fn entry_mut(
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
            self.map.load_chunk(index).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Entry {
    pub biome: Biome,
    pub ground: Ground,
    pub block: Block,
    pub thede: Option<thede::Id>,
    pub thede_visited: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
                    thede_visited: false,
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
