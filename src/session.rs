use crate::{
    entity::Player,
    error::Result,
    graphics::{BasicColor, GString, Style},
    input::{Event, Key, KeyEvent, ResizeEvent},
    math::plane::{Camera, Coord2, Direc, Nat},
    storage::{
        save::{SaveName, SavedGame},
        settings::{self, Settings, SettingsOption},
    },
    terminal,
    ui::{Menu, MenuOption},
};
use std::{collections::HashSet, time::Duration};
use tokio::time;

const TICK: Duration = Duration::from_millis(50);
const INPUT_WAIT: Duration = Duration::from_millis(TICK.as_millis() as u64 / 2);
const SETTINGS_REFRESH_TICKS: u128 = 32;

#[derive(Debug, Clone)]
/// Menu shown when player pauses.
enum PauseMenuOption {
    /// User asked to resume game.
    Resume,
    /// User asked to change settings.
    Settings,
    /// User asked to exit game to main menu.
    Exit,
}

impl PauseMenuOption {
    fn menu() -> Menu<Self> {
        Menu::new(
            gstring!["<> Paused <>"],
            vec![
                PauseMenuOption::Resume,
                PauseMenuOption::Settings,
                PauseMenuOption::Exit,
            ],
        )
    }

    async fn exec(
        &self,
        session: &mut Session,
        term: &terminal::Handle,
    ) -> Result<bool> {
        match self {
            PauseMenuOption::Resume => {
                session.render(term).await?;
                Ok(true)
            },

            PauseMenuOption::Settings => {
                let mut menu = SettingsOption::menu(&session.settings);
                let mut option = Some(0);
                loop {
                    option = menu
                        .select_with_cancel_and_initial(term, option)
                        .await?;
                    match option {
                        Some(chosen) => {
                            if !menu.options[chosen].exec(term).await? {
                                session.settings.apply_options(&menu.options);
                                session.settings.save().await?;
                                session.resize_camera(term.screen_size());
                                break Ok(true);
                            }
                        },
                        None => break Ok(true),
                    }
                }
            },

            PauseMenuOption::Exit => Ok(false),
        }
    }
}

impl MenuOption for PauseMenuOption {
    fn name(&self) -> GString {
        let string = match self {
            Self::Resume => "RESUME",
            Self::Settings => "SETTINGS",
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
    settings: Settings,
    message: GString,
}

impl Session {
    /// Initializes a new session from given saved game and save name.
    pub async fn new(game: SavedGame, name: SaveName) -> Result<Self> {
        let player_id = game.default_player().await?;
        let player = game.players().load(player_id).await?;
        let settings = settings::open().await?;
        Ok(Self {
            message: GString::default(),
            game,
            name,
            // dummy camera
            camera: Camera::new(
                player.head(),
                Coord2 { x: 80, y: 25 },
                Coord2 { x: 0, y: 0 },
            ),
            player,
            settings,
        })
    }

    /// The main loop of the game.
    pub async fn game_loop(&mut self, term: &terminal::Handle) -> Result<()> {
        self.resize_camera(term.screen_size());
        self.render(term).await?;

        let mut intval = time::interval(TICK);
        let mut ticks = 0u128;
        let mut stop = false;

        while !stop {
            self.render(term).await?;
            intval.tick().await;
            ticks = ticks.wrapping_add(1);

            if ticks % SETTINGS_REFRESH_TICKS == 0 {
                self.settings = settings::open().await?;
            }

            let fut = term.listen_event();
            if let Ok(res) = time::timeout(INPUT_WAIT, fut).await {
                match res? {
                    Event::Key(evt) => {
                        stop = self.handle_key_evt(evt, term).await?;
                    },
                    Event::Resize(evt) => {
                        self.handle_resize_evt(evt, term).await?;
                    },
                }
            };

            self.game.map().flush().await?;
        }

        Ok(())
    }

    async fn handle_key_evt(
        &mut self,
        evt: KeyEvent,
        term: &terminal::Handle,
    ) -> Result<bool> {
        match evt {
            KeyEvent {
                main_key: Key::Esc,
                ctrl: false,
                alt: false,
                shift: false,
            } => {
                let menu = PauseMenuOption::menu();
                let chosen = menu.select(term).await?;
                let ret = !menu.options[chosen].exec(self, term).await?;
                Ok(ret)
            },

            KeyEvent {
                main_key: Key::Char(' '),
                ctrl: false,
                alt: false,
                shift: false,
            } => {
                let maybe_point =
                    self.player.pointer().move_by_direc(self.player.facing());
                if let Some(point) = maybe_point {
                    let block = self.game.map().block(point).await?;

                    block.interact(&mut self.message, &self.game).await?;
                }
                Ok(false)
            },

            key => {
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

                Ok(false)
            },
        }
    }

    async fn handle_resize_evt(
        &mut self,
        evt: ResizeEvent,
        term: &terminal::Handle,
    ) -> Result<()> {
        self.resize_camera(evt.size);
        self.render(term).await?;
        Ok(())
    }

    /// Renders debug_stats and everything on the camera.
    async fn render(&self, term: &terminal::Handle) -> Result<()> {
        let mut screen = term.lock_screen().await;
        screen.clear(BasicColor::Black.into());
        self.render_map(&mut screen).await?;
        if self.settings.debug {
            self.render_debug_stats(&mut screen).await?;
        }
        self.render_stats(&mut screen).await?;
        Ok(())
    }

    /// Renders map in most of the screen.
    async fn render_map<'guard>(
        &self,
        screen: &mut terminal::Screen<'guard>,
    ) -> Result<()> {
        let rect = self.camera.rect();
        let mut entities = HashSet::new();

        for point in rect.lines() {
            self.game.map().thede(point, &self.game).await?;

            self.game.map().ground(point).await?.render(
                point,
                self.camera,
                screen,
            );

            self.game
                .map()
                .block(point)
                .await?
                .render(point, self.camera, screen, &self.game, &mut entities)
                .await?;
        }

        Ok(())
    }

    /// Renders statistics in the bottom of the screen.
    async fn render_stats<'guard>(
        &self,
        screen: &mut terminal::Screen<'guard>,
    ) -> Result<()> {
        screen.styled_text(
            &self.message,
            Style::new().top_margin(screen.handle().screen_size().y - 1),
        )?;
        Ok(())
    }

    /// Renders statistics in the bottom of the screen.
    async fn render_debug_stats<'guard>(
        &self,
        screen: &mut terminal::Screen<'guard>,
    ) -> Result<()> {
        let pos = self.player.head().printable_pos();
        let biome = self.game.map().biome(self.player.head()).await?;
        let thede =
            self.game.map().thede(self.player.head(), &self.game).await?;
        let string = format!(
            "Coord: {:>6}, {:<8} Biome: {:<8} Thede: {:<7} Seed: {:<16}",
            pos.x,
            pos.y,
            biome,
            thede,
            self.game.seed(),
        );
        screen.styled_text(&gstring![string], Style::new().align(1, 2))?;
        Ok(())
    }

    /// Updates the camera acording to the available size.
    fn resize_camera(&mut self, mut screen_size: Coord2<Nat>) {
        screen_size.y -= self.debug_stats_height();
        screen_size.y -= self.stats_height();
        self.camera = Camera::new(
            self.player.head(),
            screen_size,
            Coord2 { x: 0, y: self.debug_stats_height() },
        );
    }

    fn stats_height(&self) -> Nat {
        1
    }

    fn debug_stats_height(&self) -> Nat {
        if self.settings.debug {
            1
        } else {
            0
        }
    }
}
