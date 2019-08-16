use crate::{
    map::Map,
    orient::{Direc, IntPos},
};

#[derive(Debug)]
pub struct GameSession {
    objs_map: Map,
    player: Player,
}

#[derive(Debug)]
pub struct Player {
    x: IntPos,
    y: IntPos,
    facing: Direc,
}
