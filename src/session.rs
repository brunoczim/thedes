use crate::{
    backend::Backend,
    map::{Action, Map},
    orient::{Camera, Coord2D, Direc, Positioned, Rect},
    render::{Context, Render, RenderCore},
};
use std::{
    fmt::{self, Write},
    io,
};

/// An ongoing game.
#[derive(Debug)]
pub struct GameSession {
    camera: Camera,
    map: Map,
    player: Player,
}

impl GameSession {
    /// Creates a new game session with a camera made out of a given screen
    /// size.
    pub fn new(screen_size: Coord2D) -> Self {
        let mut this = Self {
            camera: Camera {
                rect: Rect {
                    start: Coord2D::from_map(|axis| {
                        Coord2D::ORIGIN[axis] - screen_size[axis] / 2
                    }),
                    size: screen_size,
                },
            },
            map: Map::new(),
            player: Player { pos: Coord2D::ORIGIN, facing: Direc::Up },
        };

        this.map.insert(Rect {
            start: this.player.pos,
            size: Coord2D { x: 1, y: 2 },
        });

        this
    }

    /// Moves the player in the given direction, re-drawing if needed.
    pub fn move_player<B>(
        &mut self,
        direc: Direc,
        backend: &mut B,
    ) -> io::Result<()>
    where
        B: Backend,
    {
        self.player.move_direc(direc, &mut self.map, backend, self.camera)
    }
}

#[derive(Debug)]
struct Player {
    pos: Coord2D,
    facing: Direc,
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
            Direc::Up => "ʌ\nO",
            Direc::Left => "<O",
            Direc::Down => "O\nV",
            Direc::Right => "O>",
        })
    }
}

impl Player {
    fn move_direc<B>(
        &mut self,
        direc: Direc,
        map: &mut Map,
        backend: &mut B,
        camera: Camera,
    ) -> io::Result<()>
    where
        B: Backend,
    {
        self.clear(map, camera, backend)?;

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

        self.render(map, camera, backend)?;

        Ok(())
    }
}
