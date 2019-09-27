use crate::{
    backend::Backend,
    map::{Action, Map},
    orient::{
        Axis,
        Camera,
        Coord,
        Coord2D,
        Direc,
        ICoord,
        Positioned,
        Rect,
        Vec2D,
        ORIGIN_EXCESS,
    },
    render::{Context, Render, RenderCore},
};
use std::{
    fmt::{self, Write},
    io,
};

const STATUS_HEIGHT: Coord = 4;

/// An ongoing game.
#[derive(Debug)]
pub struct GameSession {
    screen: Coord2D,
    camera: Camera,
    map: Map,
    player: Player,
}

impl GameSession {
    /// Creates a new game session with a camera made out of a given screen
    /// size.
    pub fn new(screen_size: Coord2D) -> Self {
        let player = Player { pos: Coord2D::ORIGIN, facing: Direc::Up };
        let mut this = Self {
            screen: screen_size,
            map: Map::new(),
            camera: Camera::default(),
            player,
        };

        this.map.insert(Rect {
            start: this.player.pos,
            size: Coord2D { x: 1, y: 2 },
        });

        this.make_camera();

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
        self.player.clear(&self.map, self.camera, backend)?;
        self.player.move_direc(direc, &mut self.map);
        self.update_camera();
        self.player.render(&self.map, self.camera, backend)?;
        self.render_status(backend)?;
        Ok(())
    }

    /// Handles the event in which the screen is resized.
    pub fn resize_screen<B>(
        &mut self,
        size: Coord2D,
        backend: &mut B,
    ) -> io::Result<()>
    where
        B: Backend,
    {
        self.screen = size;
        self.render_all(backend)?;
        Ok(())
    }

    /// Renders everything. Should only be called for a first render or when an
    /// event invalidates previous draws.
    pub fn render_all<B>(&mut self, backend: &mut B) -> io::Result<()>
    where
        B: Backend,
    {
        self.make_camera();
        backend.clear_screen()?;
        self.player.render(&self.map, self.camera, backend)?;
        self.render_status_bar(backend)?;
        self.render_status(backend)?;
        Ok(())
    }

    fn update_camera(&mut self) -> bool {
        let player = self.map.at(self.player.pos);
        if self.camera.rect.overlapped(player) != Some(player) {
            for axis in Axis::iter() {
                if player.start[axis] < self.camera.rect.start[axis] {
                    self.camera.rect.start[axis] = player.start[axis];
                } else if player.end()[axis] > self.camera.rect.end()[axis] {
                    self.camera.rect.start[axis] =
                        player.end()[axis] - self.camera.rect.size[axis];
                }
            }

            true
        } else {
            false
        }
    }

    fn render_status_bar<B>(&mut self, backend: &mut B) -> io::Result<()>
    where
        B: Backend,
    {
        let height = self.screen.y - STATUS_HEIGHT;
        backend.goto(Coord2D { x: 0, y: height })?;
        for _ in 0 .. self.screen.x {
            write!(backend, "—")?;
        }
        Ok(())
    }

    fn render_status<B>(&mut self, backend: &mut B) -> io::Result<()>
    where
        B: Backend,
    {
        let height = self.screen.y - STATUS_HEIGHT + 1;
        backend.goto(Coord2D { x: 0, y: height })?;
        for _ in 0 .. self.screen.x {
            write!(backend, " ")?;
        }
        backend.goto(Coord2D { x: 0, y: height })?;
        let pos = self.player.printable_pos();
        write!(backend, "{}, {}", pos.x, pos.y)?;
        Ok(())
    }

    fn make_camera(&mut self) {
        let correction = Coord2D { x: 0, y: STATUS_HEIGHT };
        self.camera = Camera {
            rect: Rect {
                start: Coord2D::from_map(|axis| {
                    self.player.pos[axis]
                        - (self.screen[axis] - correction[axis]) / 2
                }),
                size: self.screen - correction,
            },
        };
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
            Direc::Up => "Ʌ\nO",
            Direc::Left => "<O",
            Direc::Down => "O\nV",
            Direc::Right => "O>",
        })
    }
}

impl Player {
    fn center_pos(&self) -> Coord2D {
        match self.facing {
            Direc::Down | Direc::Right => self.pos,
            Direc::Up => Coord2D { y: self.pos.y + 1, ..self.pos },
            Direc::Left => Coord2D { x: self.pos.x + 1, ..self.pos },
        }
    }

    fn printable_pos(&self) -> Vec2D<ICoord> {
        Vec2D {
            x: self.center_pos().x.wrapping_sub(ORIGIN_EXCESS) as ICoord,
            y: ORIGIN_EXCESS.wrapping_sub(self.center_pos().y) as ICoord,
        }
    }

    fn move_direc(&mut self, direc: Direc, map: &mut Map) {
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
