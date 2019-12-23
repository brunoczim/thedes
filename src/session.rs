use crate::{
    backend::Backend,
    error::GameResult,
    key::Key,
    map::PhysicalMap,
    orient::{Axis, Camera, Coord, Coord2D, Direc, Rect},
    player::Player,
    rand::Seed,
    render::Render,
    storage::Save,
    term::{self, Terminal},
    ui::{Menu, MenuItem},
};
use ahash::AHasher;
use rand::{rngs::StdRng, Rng, SeedableRng as _};
use std::{
    hash::{Hash, Hasher},
    io::Write,
    slice,
};

const STATUS_HEIGHT: Coord = 4;
const POSITION_WIDTH: Coord = 14;
const POSITION_SEED_PADDING: Coord = 5;
#[allow(dead_code)]
const SEED_WIDTH: Coord = 26;

/// An ongoing game.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GameSession {
    seed: Seed,
    screen: Coord2D,
    camera: Camera,
    physical_map: PhysicalMap,
    player: Player,
    save: Option<String>,
}

impl GameSession {
    /// Creates a new game session based on the screen size of given term.
    pub fn new<B>(term: &mut Terminal<B>) -> GameResult<Self>
    where
        B: Backend,
    {
        let player = Player { pos: Coord2D::ORIGIN, facing: Direc::Up };
        let mut this = Self {
            seed: Seed::random(),
            screen: term.screen_size(),
            physical_map: PhysicalMap::new(),
            camera: Camera::default(),
            save: None,
            player,
        };

        /*this.map.insert(Rect {
            start: this.player.pos,
            size: Coord2D { x: 1, y: 2 },
        });*/

        this.make_camera();

        Ok(this)
    }

    /// The seed used by this game session.
    pub fn seed(&self) -> Seed {
        self.seed
    }

    /// Returns the save file name.
    pub fn save_name(&self) -> Option<&str> {
        self.save.as_ref().map(|s| &**s)
    }

    /// Renames the save file name.
    pub fn rename_save(&mut self, name: String) {
        self.save = Some(name);
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
                    Key::Esc => {
                        if !self.pause(term)? {
                            return Ok(term::Stop(()));
                        }

                        self.render_all(term)?;
                    },
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
        /*
        self.player.clear(&self.map, self.camera, term)?;
        self.player.move_direc(direc, &mut self.map);
        self.update_camera();
        self.player.render(&self.map, self.camera, term)?;
        self.render_mut_status_parts(term)?;
        */
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
        /*self.make_camera();
        term.clear_screen()?;
        self.player.render(&self.map, self.camera, term)?;
        self.render_status_bar(term)?;
        self.render_status(term)?;*/
        Ok(())
    }

    fn update_camera(&mut self) -> bool {
        /*let player = self.map.at(self.player.pos);
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
        }*/

        unimplemented!()
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
        write!(term, "seed: {}", self.seed.bits())?;

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

    fn pause<B>(&mut self, term: &mut Terminal<B>) -> GameResult<bool>
    where
        B: Backend,
    {
        term.call(|term| match PauseMenu.select(term)? {
            PauseMenuItem::Resume => Ok(term::Stop(true)),
            PauseMenuItem::Quit => Ok(term::Stop(false)),
            PauseMenuItem::Save => {
                Save { session: &mut *self }.save_from_user(term)?;
                Ok(term::Continue)
            },
        })
    }
}

/// An item of the menu displayed when a game session is paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuItem {
    /// Resume game.
    Resume,
    /// Save the current game.
    Save,
    /// Quit the current game.
    Quit,
}

impl MenuItem for PauseMenuItem {
    fn name(&self) -> &str {
        match self {
            PauseMenuItem::Resume => "RESUME",
            PauseMenuItem::Save => "SAVE",
            PauseMenuItem::Quit => "QUIT",
        }
    }
}

/// A menu displayed when the game session is paused.
#[derive(Debug)]
pub struct PauseMenu;

impl<'menu> Menu<'menu> for PauseMenu {
    type Item = PauseMenuItem;
    type Iter = slice::Iter<'menu, Self::Item>;

    fn title(&self) -> &str {
        "Pause Menu"
    }

    fn items(&'menu self) -> Self::Iter {
        [PauseMenuItem::Resume, PauseMenuItem::Save, PauseMenuItem::Quit].iter()
    }
}
