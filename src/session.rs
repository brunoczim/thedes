use crate::{
    backend::Backend,
    map::{Action, Map},
    orient::{Coord2D, Direc},
    render::{Context, Render},
};
use std::fmt::{self, Write};

#[derive(Debug)]
pub struct GameSession {
    objs_map: Map,
    player: Player,
}

impl GameSession {
    pub fn new() -> Self {
        Self {
            objs_map: Map::new(),
            player: Player { pos: Coord2D::ORIGIN, facing: Direc::Up },
        }
    }
}

#[derive(Debug)]
struct Player {
    pos: Coord2D,
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

impl Player {
    pub fn move_direc<B>(
        &mut self,
        direc: Direc,
        map: &mut Map,
        ctx: &mut Context<B>,
    ) -> fmt::Result
    where
        B: Backend,
    {
        self.clear(ctx)?;

        let success = match (self.facing, direc) {
            (Direc::Up, Direc::Left) => map.transaction(
                &mut self.pos,
                &[
                    Action::ShrinkY(1),
                    Action::MoveDown(1),
                    Action::MoveLeft(1),
                    Action::GrowX(1),
                ],
            ),
            (Direc::Up, Direc::Right) => map.transaction(
                &mut self.pos,
                &[Action::ShrinkY(1), Action::MoveDown(1), Action::GrowX(1)],
            ),
            (Direc::Up, Direc::Down) => {
                map.transaction(&mut self.pos, &[Action::MoveDown(1)])
            },

            (Direc::Down, Direc::Left) => map.transaction(
                &mut self.pos,
                &[Action::ShrinkY(1), Action::MoveLeft(1), Action::GrowX(1)],
            ),
            (Direc::Down, Direc::Right) => map.transaction(
                &mut self.pos,
                &[Action::ShrinkY(1), Action::GrowX(1)],
            ),
            (Direc::Down, Direc::Up) => {
                map.transaction(&mut self.pos, &[Action::MoveUp(1)])
            },

            (Direc::Left, Direc::Up) => map.transaction(
                &mut self.pos,
                &[
                    Action::ShrinkX(1),
                    Action::MoveRight(1),
                    Action::MoveUp(1),
                    Action::GrowX(1),
                ],
            ),
            (Direc::Left, Direc::Down) => map.transaction(
                &mut self.pos,
                &[Action::ShrinkX(1), Action::MoveRight(1), Action::GrowY(1)],
            ),
            (Direc::Left, Direc::Right) => {
                map.transaction(&mut self.pos, &[Action::MoveRight(1)])
            },

            (Direc::Right, Direc::Up) => map.transaction(
                &mut self.pos,
                &[Action::ShrinkX(1), Action::MoveUp(1), Action::GrowX(1)],
            ),
            (Direc::Right, Direc::Down) => map.transaction(
                &mut self.pos,
                &[Action::ShrinkX(1), Action::GrowY(1)],
            ),
            (Direc::Right, Direc::Left) => {
                map.transaction(&mut self.pos, &[Action::MoveLeft(1)])
            },

            _ => map.move_by_direc(&mut self.pos, direc),
        };

        if success {
            self.facing = direc;
        }

        self.render(ctx)
    }
}
