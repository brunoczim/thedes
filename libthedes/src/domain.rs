use gardiz::{coord::Vec2, direc::Direction};
use ndarray::{Array, Ix2};
use std::{
    collections::BTreeMap,
    ops::{Index, IndexMut},
};

pub type Coord = u16;

pub type PlayerName = String;

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
pub struct HumanLocation {
    pub head: Vec2<Coord>,
    pub facing: Direction,
}

impl HumanLocation {
    pub fn pointer(self) -> Vec2<Coord> {
        self.head.move_one(self.facing)
    }

    pub fn checked_pointer(self) -> Option<Vec2<Coord>> {
        self.head.checked_move(self.facing)
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Player {
    pub name: PlayerName,
    pub location: HumanLocation,
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
pub enum Ground {
    Grass,
    Sand,
}

impl Default for Ground {
    fn default() -> Self {
        Self::Grass
    }
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
pub enum Biome {
    Plains,
    Desert,
}

impl Default for Biome {
    fn default() -> Self {
        Self::Plains
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct MapCell {
    pub player: Option<Player>,
    pub ground: Ground,
    pub biome: Biome,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Map {
    matrix: Array<MapCell, Ix2>,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            matrix: Array::default([
                usize::from(Self::SIZE.y),
                usize::from(Self::SIZE.x),
            ]),
        }
    }
}

impl Map {
    pub const SIZE: Vec2<Coord> = Vec2 { x: 256, y: 256 };

    pub fn generate<F>(mut generator: F) -> Self
    where
        F: FnMut(Vec2<Coord>) -> MapCell,
    {
        Self {
            matrix: Array::from_shape_fn(
                [usize::from(Self::SIZE.y), usize::from(Self::SIZE.x)],
                |(y, x)| generator(Vec2 { x: x as Coord, y: y as Coord }),
            ),
        }
    }
}

impl Index<Vec2<Coord>> for Map {
    type Output = MapCell;

    fn index(&self, index: Vec2<Coord>) -> &Self::Output {
        &self.matrix[[usize::from(index.y), usize::from(index.x)]]
    }
}

impl IndexMut<Vec2<Coord>> for Map {
    fn index_mut(&mut self, index: Vec2<Coord>) -> &mut Self::Output {
        &mut self.matrix[[usize::from(index.y), usize::from(index.x)]]
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct GameSnapshot {
    pub map: Map,
    pub players: BTreeMap<PlayerName, Player>,
}
