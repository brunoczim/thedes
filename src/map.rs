use crate::{
    orient::{Axis, Coord, Coord2D, Direc, Rect},
    rand::{BlockDistr, Seed},
};
use rand::Rng;
use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, Visitor},
    ser::{Serialize, SerializeMap, Serializer},
};
use std::{collections::HashMap, fmt};

pub const CHUNK_SIZE: Coord2D = Coord2D { x: 256, y: 256 };

#[derive(Clone)]
pub struct Chunk {
    cells: [[Cell; CHUNK_SIZE.x as usize]; CHUNK_SIZE.y as usize],
}

impl fmt::Debug for Chunk {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Chunk").field("cells", &self.cells as &[_]).finish()
    }
}

#[derive(Debug)]
pub struct Cell {
    physical: PhysicalCell,
    entity: EntityId,
}

/// An action during an transaction.
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
pub enum Action {
    MoveUp(Coord),
    MoveDown(Coord),
    MoveLeft(Coord),
    MoveRight(Coord),
    GrowX(Coord),
    GrowY(Coord),
    ShrinkX(Coord),
    ShrinkY(Coord),
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Block {
    Empty,
    Wall,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum PhysicalCell {
    Block(Block),
    EntityHead(char),
    EntityPointer(char),
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
pub struct PhysicalMap {
    modified: HashMap<Coord2D, PhysicalCell>,
}

impl PhysicalMap {
    /// Creates a new unmodified map.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cell_at(&self, pos: Coord2D, seed: Seed) -> PhysicalCell {
        self.modified.get(&pos).map(|&cell| cell).unwrap_or_else(|| {
            let mut rng = seed.make_rng(pos);
            PhysicalCell::Block(rng.sample(BlockDistr))
        })
    }
}

#[cfg(test)]
mod test {
    use super::Map;
    use crate::orient::{Coord2D, Rect};

    #[test]
    fn insert_and_get() {
        let mut map = Map::new();
        let node1 = Rect {
            start: Coord2D { x: 0, y: 2 },
            size: Coord2D { x: 5, y: 5 },
        };
        let node2 = Rect {
            start: Coord2D { x: 20, y: 15 },
            size: Coord2D { x: 6, y: 4 },
        };
        let node3 = Rect {
            start: Coord2D { x: 0, y: 8 },
            size: Coord2D { x: 5, y: 5 },
        };
        let node4 = Rect {
            start: Coord2D { x: 6, y: 2 },
            size: Coord2D { x: 5, y: 7 },
        };

        assert!(map.insert(node1));
        assert_eq!(map.at(node1.start), node1);

        assert!(map.insert(node2));
        assert_eq!(map.at(node2.start), node2);
        assert_eq!(map.at(node1.start), node1);

        assert!(map.insert(node3));
        assert_eq!(map.at(node3.start), node3);

        assert!(map.insert(node4));
        assert_eq!(map.at(node4.start), node4);
    }

    #[test]
    fn insert_fails() {
        let mut map = Map::new();
        let node1 = Rect {
            start: Coord2D { x: 0, y: 2 },
            size: Coord2D { x: 5, y: 5 },
        };
        let node2 = Rect {
            start: Coord2D { x: 2, y: 2 },
            size: Coord2D { x: 6, y: 4 },
        };
        let node3 = Rect {
            start: Coord2D { x: 1, y: 3 },
            size: Coord2D { x: 6, y: 8 },
        };

        assert!(map.insert(node1));
        assert_eq!(map.at(node1.start), node1);

        assert!(!map.insert(node2));
        assert_eq!(map.try_at(node2.start), None);
        assert_eq!(map.at(node1.start), node1);

        assert!(!map.insert(node3));
        assert_eq!(map.try_at(node3.start), None);
    }

    #[test]
    fn moving() {
        let mut map = Map::new();
        let mut node1 = Rect {
            start: Coord2D { x: 0, y: 2 },
            size: Coord2D { x: 5, y: 5 },
        };
        let node2 = Rect {
            start: Coord2D { x: 0, y: 15 },
            size: Coord2D { x: 6, y: 4 },
        };

        assert!(map.insert(node1));
        assert!(map.insert(node2));

        assert!(!map.move_vert(&mut node1.start, 17));
        assert!(!map.move_vert(&mut node1.start, 30));
        assert!(!map.move_vert(&mut node1.start, 12));

        assert!(map.move_vert(&mut node1.start, 0));

        assert!(map.move_horz(&mut node1.start, 20));

        assert!(map.move_vert(&mut node1.start, 15));

        assert!(!map.move_horz(&mut node1.start, 0));
        assert!(!map.move_horz(&mut node1.start, 5));
    }

    #[test]
    fn resizing() {
        let mut map = Map::new();
        let mut node1 = Rect {
            start: Coord2D { x: 0, y: 2 },
            size: Coord2D { x: 5, y: 5 },
        };
        let mut node2 = Rect {
            start: Coord2D { x: 0, y: 15 },
            size: Coord2D { x: 6, y: 4 },
        };

        assert!(map.insert(node1));
        assert!(map.insert(node2));

        node1.size.y = 20;
        assert!(!map.resize(node1));
        node1.size.y = 10;
        assert!(map.resize(node1));

        assert!(map.move_horz(&mut node1.start, 15));
        assert!(map.move_vert(&mut node1.start, 15));

        node2.size.x = 20;
        assert!(!map.resize(node2));
        node2.size.x = 10;
        assert!(map.resize(node2));
    }
}
