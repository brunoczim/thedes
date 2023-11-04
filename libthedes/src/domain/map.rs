use std::ops::{Index, IndexMut};

use gardiz::{coord::Vec2, rect::Rect};
use ndarray::{Array, Ix2};
use num::CheckedSub;

use super::{plane::Coord, player};

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
pub struct Cell {
    pub player: player::OptionalName,
    pub ground: Ground,
    pub biome: Biome,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Slice {
    offset: Vec2<Coord>,
    matrix: Array<Cell, Ix2>,
}

impl Slice {
    pub fn generate<F>(view: Rect<Coord>, mut generator: F) -> Self
    where
        F: FnMut(Vec2<Coord>) -> Cell,
    {
        Self {
            offset: view.start,
            matrix: Array::from_shape_fn(
                [usize::from(view.size.y), usize::from(view.size.x)],
                |(y, x)| {
                    generator(Vec2 {
                        x: x as Coord + view.start.x,
                        y: y as Coord + view.start.y,
                    })
                },
            ),
        }
    }

    pub fn default(view: Rect<Coord>) -> Self {
        Self {
            offset: view.start,
            matrix: Array::default([
                usize::from(view.size.y),
                usize::from(view.size.x),
            ]),
        }
    }

    pub fn view(&self) -> Rect<Coord> {
        Rect {
            start: self.offset,
            size: Vec2 {
                y: self.matrix.dim().0 as Coord,
                x: self.matrix.dim().1 as Coord,
            },
        }
    }

    pub fn sub(&self, view: Rect<Coord>) -> Option<Self> {
        if self.view().has_point(view.end_inclusive())
            && self.view().has_point(view.start)
        {
            Some(Self {
                offset: view.start,
                matrix: self
                    .matrix
                    .slice(ndarray::s![
                        usize::from(view.start.y)
                            ..= usize::from(view.end_inclusive().y),
                        usize::from(view.start.x)
                            ..= usize::from(view.end_inclusive().x)
                    ])
                    .to_owned(),
            })
        } else {
            None
        }
    }

    pub fn get(&self, index: Vec2<Coord>) -> Option<&Cell> {
        let shifted = index.checked_sub(&self.offset)?;
        self.matrix.get([usize::from(shifted.y), usize::from(shifted.x)])
    }

    pub fn get_mut(&mut self, index: Vec2<Coord>) -> Option<&mut Cell> {
        let shifted = index.checked_sub(&self.offset)?;
        self.matrix.get_mut([usize::from(shifted.y), usize::from(shifted.x)])
    }
}

impl Index<Vec2<Coord>> for Slice {
    type Output = Cell;

    fn index(&self, index: Vec2<Coord>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IndexMut<Vec2<Coord>> for Slice {
    fn index_mut(&mut self, index: Vec2<Coord>) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
