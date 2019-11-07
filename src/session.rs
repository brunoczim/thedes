use crate::{
    backend::{check_screen_size, Backend},
    error::GameResult,
    key::Key,
    map::Map,
    orient::{Axis, Camera, Coord, Coord2D, Direc, Rect},
    player::Player,
    render::Render,
    timer,
};
use rand::Rng as _;
use std::time::Duration;

const STATUS_HEIGHT: Coord = 4;
const POSITION_WIDTH: Coord = 14;
const POSITION_SEED_PADDING: Coord = 5;
#[allow(dead_code)]
const SEED_WIDTH: Coord = 26;

type Seed = u64;

/// An ongoing game.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GameSession {
    seed: Seed,
    screen: Coord2D,
    camera: Camera,
    map: Map,
    player: Player,
}

impl GameSession {
    /// Creates a new game session based on the screen size of given backend.
    pub fn new<B>(backend: &mut B) -> GameResult<Self>
    where
        B: Backend,
    {
        let player = Player { pos: Coord2D::ORIGIN, facing: Direc::Up };
        let mut this = Self {
            seed: rand::thread_rng().gen(),
            screen: backend.screen_size()?,
            map: Map::new(),
            camera: Camera::default(),
            player,
        };

        this.map.insert(Rect {
            start: this.player.pos,
            size: Coord2D { x: 1, y: 2 },
        });

        this.make_camera();

        Ok(this)
    }

    /// Runs an entire game session.
    pub fn exec<B>(&mut self, backend: &mut B) -> GameResult<()>
    where
        B: Backend,
    {
        let mut screen_size = self.screen;
        self.render_all(backend)?;
        timer::tick(Duration::from_millis(50), move || {
            if check_screen_size(backend, &mut screen_size)? {
                self.resize_screen(screen_size, backend)?;
            }

            if let Some(key) = backend.try_get_key()? {
                match key {
                    Key::Up => self.move_player(Direc::Up, backend)?,
                    Key::Down => self.move_player(Direc::Down, backend)?,
                    Key::Left => self.move_player(Direc::Left, backend)?,
                    Key::Right => self.move_player(Direc::Right, backend)?,
                    Key::Char('q') => return Ok(timer::Stop(())),
                    _ => (),
                }
            }

            Ok(timer::Continue)
        })
    }

    /// Moves the player in the given direction, re-drawing if needed.
    pub fn move_player<B>(
        &mut self,
        direc: Direc,
        backend: &mut B,
    ) -> GameResult<()>
    where
        B: Backend,
    {
        self.player.clear(&self.map, self.camera, backend)?;
        self.player.move_direc(direc, &mut self.map);
        self.update_camera();
        self.player.render(&self.map, self.camera, backend)?;
        self.render_mut_status_parts(backend)?;
        Ok(())
    }

    /// Handles the event in which the screen is resized.
    pub fn resize_screen<B>(
        &mut self,
        size: Coord2D,
        backend: &mut B,
    ) -> GameResult<()>
    where
        B: Backend,
    {
        self.screen = size;
        self.render_all(backend)?;
        Ok(())
    }

    /// Renders everything. Should only be called for a first render or when an
    /// event invalidates previous draws.
    pub fn render_all<B>(&mut self, backend: &mut B) -> GameResult<()>
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

    fn render_status_bar<B>(&mut self, backend: &mut B) -> GameResult<()>
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

    fn render_mut_status_parts<B>(&mut self, backend: &mut B) -> GameResult<()>
    where
        B: Backend,
    {
        let height = self.screen.y - STATUS_HEIGHT + 1;

        backend.goto(Coord2D { x: 0, y: height })?;
        for _ in 0 .. POSITION_WIDTH {
            write!(backend, " ")?;
        }
        backend.goto(Coord2D { x: 0, y: height })?;
        let pos = self.player.center_pos().printable_pos();
        write!(backend, "{}, {}", pos.x, pos.y)?;

        Ok(())
    }

    fn render_status<B>(&mut self, backend: &mut B) -> GameResult<()>
    where
        B: Backend,
    {
        self.render_mut_status_parts(backend)?;

        let height = self.screen.y - STATUS_HEIGHT + 1;
        backend.goto(Coord2D {
            x: POSITION_WIDTH + POSITION_SEED_PADDING,
            y: height,
        })?;
        write!(backend, "seed: {}", self.seed)?;

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
