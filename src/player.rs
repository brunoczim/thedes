use crate::{
    backend::Backend,
    map::{Action, Map},
    orient::{Coord2D, Direc, Positioned},
    render::{Context, RenderCore},
};
use std::fmt::{self, Write};

/// A handle to the player.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Player {
    /// Top-left position of this player.
    pub pos: Coord2D,
    /// Direction to which the player is facing.
    pub facing: Direc,
}

impl Positioned for Player {
    fn top_left(&self) -> Coord2D {
        self.pos
    }
}

impl RenderCore for Player {
    fn render_raw<B>(&self, ctx: &mut Context<B>) -> fmt::Result
    where
        B: Backend,
    {
        ctx.write_str(match self.facing {
            Direc::Up => "Ʌ\nO",
            Direc::Left => "<O",
            Direc::Down => "O\nV",
            Direc::Right => "O>",
        })
    }
}

impl Player {
    /// Returns the "center" position of the player, i.e. the position of the
    /// player's "body".
    pub fn center_pos(&self) -> Coord2D {
        match self.facing {
            Direc::Down | Direc::Right => self.pos,
            Direc::Up => Coord2D { y: self.pos.y + 1, ..self.pos },
            Direc::Left => Coord2D { x: self.pos.x + 1, ..self.pos },
        }
    }

    /// Moves the player at the given direction, by one step, either just
    /// turning the player around or actually walking.
    pub fn move_direc(&mut self, direc: Direc, map: &mut Map) {
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
                    Action::GrowY(1),
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
                &[Action::ShrinkX(1), Action::MoveUp(1), Action::GrowY(1)],
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
    }
}
