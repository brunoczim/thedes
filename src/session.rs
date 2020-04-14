use crate::{
    coord::{Camera, Coord2, Direc, Nat},
    entity::Player,
    error::Result,
    graphics::{Color, Color2, GString, Grapheme, Style, Tile},
    input::{Event, Key, KeyEvent},
    storage::save::{SaveName, SavedGame},
    terminal,
    ui::{Menu, MenuOption},
};
use std::{collections::HashSet, time::Duration};
use tokio::time;

const TICK: Duration = Duration::from_millis(50);
const INPUT_WAIT: Duration = Duration::from_millis(TICK.as_millis() as u64 / 2);
const STATS_HEIGHT: Nat = 2;

#[derive(Debug)]
/// Menu shown when player pauses.
pub enum PauseMenuOption {
    /// User asked to resume game.
    Resume,
    /// User asked to exit game to main menu.
    Exit,
}

impl PauseMenuOption {
    pub fn menu() -> Menu<Self> {
        Menu::new(
            gstring!["<> Paused <>"],
            vec![PauseMenuOption::Resume, PauseMenuOption::Exit],
        )
    }
}

impl MenuOption for PauseMenuOption {
    fn name(&self) -> GString {
        let string = match self {
            Self::Resume => "RESUME",
            Self::Exit => "EXIT TO MAIN MENU",
        };

        gstring![string]
    }
}

/// A struct containing everything about the game session.
#[derive(Debug)]
pub struct Session {
    game: SavedGame,
    name: SaveName,
    player: Player,
    camera: Camera,
}

impl Session {
    /// Initializes a new session from given saved game and save name.
    pub async fn new(game: SavedGame, name: SaveName) -> Result<Self> {
        let player_id = game.default_player().await?;
        let player = game.players().load(player_id).await?;
        Ok(Self {
            game,
            name,
            // dummy camera
            camera: Camera::new(
                player.head(),
                Coord2 { x: 80, y: 25 },
                Coord2 { x: 0, y: STATS_HEIGHT },
            ),
            player,
        })
    }

    /// The main loop of the game.
    pub async fn game_loop(&mut self, term: &terminal::Handle) -> Result<()> {
        self.resize_camera(term.screen_size());
        self.render(term).await?;

        let mut intval = time::interval(TICK);

        loop {
            self.render(term).await?;
            intval.tick().await;

            let fut = term.listen_event();
            let evt = match time::timeout(INPUT_WAIT, fut).await {
                Ok(res) => res?,
                Err(_) => continue,
            };

            match evt {
                Event::Key(KeyEvent {
                    main_key: Key::Esc,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => match PauseMenuOption::menu().select(term).await? {
                    PauseMenuOption::Resume => self.render(term).await?,
                    PauseMenuOption::Exit => break Ok(()),
                },

                Event::Key(key) => {
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

                Event::Resize(evt) => {
                    self.resize_camera(evt.size);
                    self.render(term).await?;
                },
            }
        }
    }

    /// Renders stats and everything on the camera.
    async fn render(&self, term: &terminal::Handle) -> Result<()> {
        let mut screen = term.lock_screen().await;
        screen.clear(Color::Black);
        self.render_map(&mut screen).await?;
        self.render_stats(&mut screen)?;
        Ok(())
    }

    /// Renders map in most of the screen.
    async fn render_map<'guard>(
        &self,
        screen: &mut terminal::Screen<'guard>,
    ) -> Result<()> {
        let rect = self.camera.rect();
        let mut entities = HashSet::new();

        for x in rect.start.x .. rect.end().x {
            for y in rect.start.y .. rect.end().y {
                let coord = Coord2 { x, y };
                let ground = self.game.grounds().get(coord).await?;
                ground.render(coord, self.camera, screen);
                let block = self.game.blocks().get(coord).await?;
                let fut = block.render(
                    coord,
                    self.camera,
                    screen,
                    &self.game,
                    &mut entities,
                );
                fut.await?;
            }
        }

        Ok(())
    }

    /// Renders statistics in the bottom of the screen.
    fn render_stats<'guard>(
        &self,
        screen: &mut terminal::Screen<'guard>,
    ) -> Result<()> {
        let grapheme = Grapheme::new_lossy("—");
        for x in 0 .. screen.handle().screen_size().x {
            let tile =
                Tile { grapheme: grapheme.clone(), colors: Color2::default() };
            screen.set(Coord2 { x, y: 1 }, tile);
        }
        let pos = self.player.head().printable_pos();
        let string = format!("Coord: {}, {}", pos.x, pos.y);
        screen.styled_text(&gstring![string], Style::new())?;
        Ok(())
    }

    /// Updates the camera acording to the available size.
    fn resize_camera(&mut self, mut screen_size: Coord2<Nat>) {
        screen_size.y -= STATS_HEIGHT;
        self.camera = Camera::new(
            self.player.head(),
            screen_size,
            Coord2 { x: 0, y: STATS_HEIGHT },
        );
    }
}
