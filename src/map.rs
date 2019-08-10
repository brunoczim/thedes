use crate::orient::{Coord, ICoord};
use std::collections::HashMap;
use tree::Map as TreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node {
    pub x: ICoord,
    pub y: ICoord,
    pub width: Coord,
    pub height: Coord,
}

pub struct Map {
    xy: TreeMap<(ICoord, ICoord), Node>,
    yx: TreeMap<(ICoord, ICoord), Node>,
}
