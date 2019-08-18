use crate::{
    backend::Backend,
    map::Map,
    orient::{Coord, Direc},
    render::{Context, Render},
};
use std::fmt::{self, Write};

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

impl Render for Player {
    fn render<B>(&self, ctx: &mut Context<B>) -> fmt::Result
    where
        B: Backend,
    {
        ctx.write_str(match self.facing {
            Direc::Up => "^\nO",
            Direc::Left => "<O",
            Direc::Down => "O\nV",
            Direc::Right => "O>",
        })
    }
}
