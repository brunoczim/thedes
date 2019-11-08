use crate::{
    backend::Backend,
    error::GameResult,
    key::Key,
    map::Map,
    orient::{Axis, Camera, Coord, Coord2D, Direc, Rect},
    player::Player,
    render::Render,
    term::{self, Terminal},
};
use rand::Rng as _;
use std::io::Write;

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
    /// Creates a new game session based on the screen size of given term.
    pub fn new<B>(term: &mut Terminal<B>) -> GameResult<Self>
    where
        B: Backend,
    {
        let player = Player { pos: Coord2D::ORIGIN, facing: Direc::Up };
        let mut this = Self {
            seed: rand::thread_rng().gen(),
            screen: term.screen_size(),
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
    pub fn exec<B>(&mut self, term: &mut Terminal<B>) -> GameResult<()>
    where
        B: Backend,
    {
        self.render_all(term)?;
        term.call(move |term| {
            if term.has_resized() {
                self.resize_screen(term)?;
            }

            if let Some(key) = term.key()? {
                match key {
                    Key::Up => self.move_player(Direc::Up, term)?,
                    Key::Down => self.move_player(Direc::Down, term)?,
                    Key::Left => self.move_player(Direc::Left, term)?,
                    Key::Right => self.move_player(Direc::Right, term)?,
                    Key::Char('q') => return Ok(term::Stop(())),
                    _ => (),
                }
            }

            Ok(term::Continue)
        })
    }

    /// Moves the player in the given direction, re-drawing if needed.
    pub fn move_player<B>(
        &mut self,
        direc: Direc,
        term: &mut Terminal<B>,
    ) -> GameResult<()>
    where
        B: Backend,
    {
        self.player.clear(&self.map, self.camera, term)?;
        self.player.move_direc(direc, &mut self.map);
        self.update_camera();
        self.player.render(&self.map, self.camera, term)?;
        self.render_mut_status_parts(term)?;
        Ok(())
    }

    /// Handles the event in which the screen is resized.
    pub fn resize_screen<B>(&mut self, term: &mut Terminal<B>) -> GameResult<()>
    where
        B: Backend,
    {
        self.screen = term.screen_size();
        self.render_all(term)?;
        Ok(())
    }

    /// Renders everything. Should only be called for a first render or when an
    /// event invalidates previous draws.
    pub fn render_all<B>(&mut self, term: &mut Terminal<B>) -> GameResult<()>
    where
        B: Backend,
    {
        self.make_camera();
        term.clear_screen()?;
        self.player.render(&self.map, self.camera, term)?;
        self.render_status_bar(term)?;
        self.render_status(term)?;
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

    fn render_status_bar<B>(&mut self, term: &mut Terminal<B>) -> GameResult<()>
    where
        B: Backend,
    {
        let height = self.screen.y - STATUS_HEIGHT;
        term.goto(Coord2D { x: 0, y: height })?;
        for _ in 0 .. self.screen.x {
            write!(term, "—")?;
        }
        Ok(())
    }

    fn render_mut_status_parts<B>(
        &mut self,
        term: &mut Terminal<B>,
    ) -> GameResult<()>
    where
        B: Backend,
    {
        let height = self.screen.y - STATUS_HEIGHT + 1;

        term.goto(Coord2D { x: 0, y: height })?;
        for _ in 0 .. POSITION_WIDTH {
            write!(term, " ")?;
        }
        term.goto(Coord2D { x: 0, y: height })?;
        let pos = self.player.center_pos().printable_pos();
        write!(term, "{}, {}", pos.x, pos.y)?;

        Ok(())
    }

    fn render_status<B>(&mut self, term: &mut Terminal<B>) -> GameResult<()>
    where
        B: Backend,
    {
        self.render_mut_status_parts(term)?;

        let height = self.screen.y - STATUS_HEIGHT + 1;
        term.goto(Coord2D {
            x: POSITION_WIDTH + POSITION_SEED_PADDING,
            y: height,
        })?;
        write!(term, "seed: {}", self.seed)?;

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
