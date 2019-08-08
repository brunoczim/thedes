use crate::orient::{ICoord, Coord};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    width: Coord,
    height: Coord,
    // horizontal predecessor's x
    horz_pred: ICoord,
    // vertical predecessor's y
    vert_pred: ICoord,
    // horizontal successor's x
    horz_succ: ICoord,
    // vertical successor's y
    vert_succ: ICoord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    nodes: HashMap<(ICoord, ICoord), Node>,
}
