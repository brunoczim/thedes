use crate::{
    map::Map,
    orient::{Coord, Direc},
};

#[derive(Debug)]
pub struct GameSession {
    objs_map: Map,
    player: Player,
}

#[derive(Debug)]
pub struct Player {
    x: Coord,
    y: Coord,
    facing: Direc,
}
