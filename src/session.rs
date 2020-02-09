use crate::{
    entity,
    error::GameResult,
    input::{Event, Key, KeyEvent},
    orient::{Camera, Coord, Coord2D, Direc},
    render::MIN_SCREEN,
    storage::save::{SaveName, SavedGame},
    terminal,
    ui::{Menu, MenuItem},
};
use std::{collections::HashSet, fmt::Write, time::Duration};
use tokio::time;

const TICK: Duration = Duration::from_millis(50);
const INPUT_WAIT: Duration = Duration::from_millis(TICK.as_millis() as u64 / 2);
const STATS_HEIGHT: Coord = 2;

#[derive(Debug)]
/// Menu shown when player pauses.
pub enum PauseMenuItem {
    Resume,
    Exit,
}

impl MenuItem for PauseMenuItem {
    fn name(&self) -> &str {
        match self {
            Self::Resume => "RESUME",
            Self::Exit => "EXIT TO MAIN MENU",
        }
    }
}

#[derive(Debug)]
/// A struct containing everything about the game session.
pub struct Session {
    game: SavedGame,
    name: SaveName,
    player: entity::Player,
    camera: Camera,
}

impl Session {
    /// Initializes a new session from given saved game and save name.
    pub async fn new(game: SavedGame, name: SaveName) -> GameResult<Self> {
        let player_id = game.player_id().await?;
        let player = game.player(player_id).await?;
        Ok(Self {
            game,
            name,
            // dummy camera
            camera: Camera::new(
                player.head(),
                MIN_SCREEN,
                Coord2D { x: 0, y: STATS_HEIGHT },
            ),
            player,
        })
    }

    /// The main loop of the game.
    pub async fn game_loop(
        &mut self,
        term: &mut terminal::Handle,
    ) -> GameResult<()> {
        self.resize_camera(term.screen_size());
        self.render(term).await?;

        let mut intval = time::interval(TICK);

        loop {
            self.render(term).await?;
            intval.tick().await;

            match time::timeout(INPUT_WAIT, term.listen_event()).await {
                Ok(Event::Key(KeyEvent {
                    main_key: Key::Esc,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => match Menu::PAUSE_MENU.select(term).await? {
                    PauseMenuItem::Resume => self.render(term).await?,
                    PauseMenuItem::Exit => break Ok(()),
                },

                Ok(Event::Key(key)) => {
                    let maybe_direc = match key {
                        KeyEvent {
                            main_key: Key::Up,
                            alt: false,
                            shift: false,
                            ..
                        } => Some(Direc::Up),

                        KeyEvent {
                            main_key: Key::Down,
                            alt: false,
                            shift: false,
                            ..
                        } => Some(Direc::Down),

                        KeyEvent {
                            main_key: Key::Left,
                            alt: false,
                            shift: false,
                            ..
                        } => Some(Direc::Left),

                        KeyEvent {
                            main_key: Key::Right,
                            alt: false,
                            shift: false,
                            ..
                        } => Some(Direc::Right),

                        _ => None,
                    };

                    if let Some(direc) = maybe_direc {
                        if key.ctrl {
                            self.player.step(direc, &self.game).await?;
                        } else {
                            self.player.move_around(direc, &self.game).await?;
                        }
                        self.camera.update(direc, self.player.head(), 6);
                    }
                },

                Ok(Event::Resize(evt)) => {
                    self.resize_camera(evt.size);
                    self.render(term).await?;
                },

                Err(_) => (),
            }
        }
    }

    /// Renders stats and everything on the camera.
    async fn render(&self, term: &mut terminal::Handle) -> GameResult<()> {
        term.clear_screen()?;
        self.render_map(term).await?;
        self.render_stats(term).await?;
        term.flush().await?;
        Ok(())
    }

    /// Renders map in most of the screen.
    async fn render_map(&self, term: &mut terminal::Handle) -> GameResult<()> {
        let rect = self.camera.rect();
        let offset = self.camera.offset();
        let mut entities = HashSet::new();

        for x in rect.start.x .. rect.end().x {
            for y in rect.start.y .. rect.end().y {
                let coord = Coord2D { x, y };
                let block = self.game.block_at(coord).await?;
                term.goto(coord - rect.start + offset)?;
                block
                    .render(self.camera, term, &self.game, &mut entities)
                    .await?;
            }
        }

        Ok(())
    }

    /// Renders statistics in the bottom of the screen.
    async fn render_stats(
        &self,
        term: &mut terminal::Handle,
    ) -> GameResult<()> {
        term.goto(Coord2D { x: 0, y: 1 })?;
        for _ in 0 .. term.screen_size().x {
            write!(term, "—")?;
        }
        term.goto(Coord2D { x: 0, y: 0 })?;
        let pos = self.player.head().printable_pos();
        write!(term, "Coord: {}, {}", pos.x, pos.y)?;
        Ok(())
    }

    /// Updates the camera acording to the available size.
    fn resize_camera(&mut self, mut screen_size: Coord2D) {
        screen_size.y -= STATS_HEIGHT;
        self.camera = Camera::new(
            self.player.head(),
            screen_size,
            Coord2D { x: 0, y: STATS_HEIGHT },
        );
    }
}
