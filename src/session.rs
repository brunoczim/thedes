use crate::{
    map::Map,
    orient::{Direc, ICoord},
};

#[derive(Debug)]
pub struct GameSession {
    objs_map: Map,
    player: Player,
}

#[derive(Debug)]
pub struct Player {
    x: ICoord,
    y: ICoord,
    facing: Direc,
}
