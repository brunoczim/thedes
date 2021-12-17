use super::{pack_point, unpack_chunk, unpack_offset, Cache, Chunk};
use crate::common::{biome::Biome, ground::Ground};
use gardiz::coord::Vec2;

#[test]
fn pack_unpack() {
    let point1 = Vec2 { x: 4857, y: 7375 };
    assert_eq!(point1, pack_point(unpack_chunk(point1), unpack_offset(point1)));
}

#[test]
fn cache() {
    let mut cache = Cache::new(4);
    let mut chunk1 = Chunk::default();
    chunk1[Vec2 { y: 0, x: 0 }].biome = Biome::Desert;
    let mut chunk2 = Chunk::default();
    chunk2[Vec2 { y: 0, x: 1 }].biome = Biome::Desert;
    let mut chunk3 = Chunk::default();
    chunk3[Vec2 { y: 1, x: 1 }].biome = Biome::Desert;
    let mut chunk4 = Chunk::default();
    chunk4[Vec2 { y: 0, x: 0 }].biome = Biome::RockDesert;
    let mut chunk5 = Chunk::default();
    chunk5[Vec2 { y: 1, x: 0 }].biome = Biome::RockDesert;
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
        .ground = Ground::Sand;
    chunk1[Vec2 { y: 0, x: 0 }].ground = Ground::Sand;

    assert_eq!(cache.chunk(Vec2 { x: 5, y: 0 }), Some(&chunk1));
    assert!(cache.needs_flush.contains(&Vec2 { x: 5, y: 0 }));

    // last = 4, 2, 1, 5 = first
    cache
        .entry_mut(pack_point(Vec2 { x: 2, y: 0 }, Vec2 { x: 0, y: 1 }))
        .unwrap()
        .ground = Ground::Rock;
    chunk5[Vec2 { y: 1, x: 0 }].ground = Ground::Rock;

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
