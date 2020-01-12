use crate::{
    entity,
    error::GameResult,
    input::{Event, Key, KeyEvent},
    orient::{Camera, Coord2D, Direc},
    render::MIN_SCREEN,
    storage::save::{SaveName, SavedGame},
    terminal,
    ui::{Menu, MenuItem},
};
use std::collections::HashSet;

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
            camera: Camera::new(player.head(), MIN_SCREEN),
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

        loop {
            match term.listen_event().await {
                Event::Key(KeyEvent {
                    main_key: Key::Esc,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => match Menu::PAUSE_MENU.select(term).await? {
                    PauseMenuItem::Resume => self.render(term).await?,
                    PauseMenuItem::Exit => break Ok(()),
                },

                Event::Key(key) => {
                    let maybe_direc = match key {
                        KeyEvent {
                            main_key: Key::Up,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        } => Some(Direc::Up),

                        KeyEvent {
                            main_key: Key::Down,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        } => Some(Direc::Down),

                        KeyEvent {
                            main_key: Key::Left,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        } => Some(Direc::Left),

                        KeyEvent {
                            main_key: Key::Right,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        } => Some(Direc::Right),

                        _ => None,
                    };

                    if let Some(direc) = maybe_direc {
                        let before = self.player.clone();
                        self.player.move_around(direc, &self.game).await?;
                        let updated =
                            self.camera.update(direc, self.player.head(), 2);
                        if updated {
                            self.render(term).await?;
                        } else if before != self.player {
                            before.clear(self.camera, term).await?;
                            self.player.render(self.camera, term).await?;
                            term.flush().await?;
                        }
                    }
                },

                Event::Resize(evt) => {
                    self.resize_camera(evt.size);
                    self.render(term).await?;
                },
            }
        }
    }

    /// Renders everything on the camera.
    async fn render(&self, term: &mut terminal::Handle) -> GameResult<()> {
        term.clear_screen()?;
        let rect = self.camera.rect();
        let mut entities = HashSet::new();

        for x in rect.start.x .. rect.end().x {
            for y in rect.start.y .. rect.end().y {
                let coord = Coord2D { x, y };
                let block = self.game.block_at(coord).await?;
                term.goto(coord - rect.start)?;
                block
                    .render(self.camera, term, &self.game, &mut entities)
                    .await?;
            }
        }

        term.flush().await?;
        Ok(())
    }

    /// Updates the camera acording to the available size.
    fn resize_camera(&mut self, screen_size: Coord2D) {
        self.camera = Camera::new(self.player.head(), screen_size);
    }
}
