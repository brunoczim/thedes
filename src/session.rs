use crate::{
    entity::Player,
    error::Result,
    map::Coord,
    storage::{
        save::{SaveName, SavedGame},
        settings::{self, Settings, SettingsOption},
    },
};
use andiskaz::{
    color::BasicColor,
    event::{Event, Key, KeyEvent, ResizeEvent},
    screen::Screen,
    string::TermString,
    style::Style,
    terminal::{Terminal, TerminalGuard},
    tstring,
    ui::menu::{Menu, MenuOption},
};
use gardiz::{coord::Vec2, direc::Direction, rect::Rect};
use num::rational::Ratio;
use std::{
    collections::HashSet,
    ops::{Add, Sub},
    time::Duration,
};
use tokio::time;

const TICK: Duration = Duration::from_millis(50);
const SETTINGS_REFRESH_TICKS: u128 = 32;
const BORDER_THRESHOLD: Ratio<Coord> = Ratio::new_raw(1, 3);

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
            tstring!["<> Paused <>"],
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
        term: &mut Terminal,
    ) -> Result<bool> {
        match self {
            PauseMenuOption::Resume => {
                let mut guard = term.lock_now().await?;
                session.resize_camera(guard.screen().size());
                session.render(guard.screen()).await?;
                Ok(true)
            },

            PauseMenuOption::Settings => {
                let mut menu = SettingsOption::menu(&session.settings);
                let mut option = Some(0);
                loop {
                    let is_cancel = option.is_none();
                    let init_option = option.unwrap_or(0);
                    option = menu
                        .select_cancel_initial(term, init_option, is_cancel)
                        .await?;

                    match option {
                        Some(chosen) => {
                            if !menu.options[chosen].exec(term).await? {
                                session.settings.apply_options(&menu.options);
                                session.settings.save().await?;
                                break Ok(false);
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
    fn name(&self) -> TermString {
        let string = match self {
            Self::Resume => "RESUME",
            Self::Settings => "SETTINGS",
            Self::Exit => "EXIT TO MAIN MENU",
        };

        tstring![string]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Action {
    Pause,
    Nop,
}

/// A struct containing everything about the game session.
#[derive(Debug)]
pub struct Session {
    game: SavedGame,
    name: SaveName,
    player: Player,
    camera: Camera,
    settings: Settings,
    message: TermString,
}

impl Session {
    /// Initializes a new session from given saved game and save name.
    pub async fn new(game: SavedGame, name: SaveName) -> Result<Self> {
        let player_id = game.default_player().await?;
        let player = game.players().load(player_id).await?;
        let settings = settings::open().await?;
        Ok(Self {
            message: TermString::default(),
            game,
            name,
            // dummy camera
            camera: Camera::new(
                player.head(),
                Vec2 { x: 80, y: 25 },
                Vec2 { x: 0, y: 0 },
            ),
            player,
            settings,
        })
    }

    /// The main loop of the game.
    pub async fn game_loop(&mut self, term: &mut Terminal) -> Result<()> {
        {
            let mut guard = term.lock_now().await?;
            self.resize_camera(guard.screen().size());
            self.render(guard.screen()).await?;
        }

        let mut intval = time::interval(TICK);
        let mut ticks = 0u128;
        let mut running = true;

        while running {
            intval.tick().await;
            ticks = ticks.wrapping_add(1);
            if ticks % SETTINGS_REFRESH_TICKS == 0 {
                self.settings = settings::open().await?;
            }
            running = self.tick(term).await?;
        }

        Ok(())
    }

    async fn tick(&mut self, term: &mut Terminal) -> Result<bool> {
        let action = {
            let mut guard = term.lock_now().await?;
            self.dispatch_event(&mut guard).await?
        };

        let running = self.dispatch_action(term, action).await?;
        self.game.map().flush().await?;
        Ok(running)
    }

    async fn dispatch_action(
        &mut self,
        term: &mut Terminal,
        action: Action,
    ) -> Result<bool> {
        match action {
            Action::Pause => {
                let menu = PauseMenuOption::menu();
                let chosen = menu.select(term).await?;
                menu.options[chosen].exec(self, term).await
            },
            Action::Nop => Ok(true),
        }
    }

    async fn dispatch_event<'term>(
        &mut self,
        term: &mut TerminalGuard<'term>,
    ) -> Result<Action> {
        match term.event() {
            Some(Event::Key(key_evt)) => {
                self.dispatch_key(key_evt, term.screen()).await
            },
            Some(Event::Resize(resize_evt)) => {
                self.dispatch_resize(resize_evt, term.screen()).await
            },
            None => self.dispatch_no_event(term.screen()).await,
        }
    }

    async fn dispatch_key<'term>(
        &mut self,
        evt: KeyEvent,
        screen: &mut Screen<'term>,
    ) -> Result<Action> {
        match evt {
            KeyEvent {
                main_key: Key::Esc,
                ctrl: false,
                alt: false,
                shift: false,
            } => self.dispatch_esc(screen).await,

            KeyEvent {
                main_key: Key::Char(' '),
                ctrl: false,
                alt: false,
                shift: false,
            } => self.dispatch_space(screen).await,

            KeyEvent {
                main_key: Key::Char('f'),
                ctrl: false,
                alt: false,
                shift: false,
            } => {
                tracing::debug!("bananas");
                Ok(Action::Nop)
            },

            key => self.dispatch_arrows(key, screen).await,
        }
    }

    async fn dispatch_esc<'term>(
        &mut self,
        _screen: &mut Screen<'term>,
    ) -> Result<Action> {
        Ok(Action::Pause)
    }

    async fn dispatch_space<'term>(
        &mut self,
        screen: &mut Screen<'term>,
    ) -> Result<Action> {
        let maybe_point =
            self.player.pointer().checked_move(self.player.facing());
        if let Some(point) = maybe_point {
            let block = self.game.map().block(point).await?;
            block.interact(&mut self.message, &self.game).await?;
            self.render(screen).await?;
        }
        Ok(Action::Nop)
    }

    async fn dispatch_arrows<'term>(
        &mut self,
        key: KeyEvent,
        screen: &mut Screen<'term>,
    ) -> Result<Action> {
        let maybe_direc = Self::key_direction(key);

        if let Some(direction) = maybe_direc {
            if key.ctrl {
                self.player.step(direction, &self.game).await?;
            } else {
                self.player.move_around(direction, &self.game).await?;
            }
            self.camera.update(direction, self.player.head(), BORDER_THRESHOLD);
            self.render(screen).await?;
        }

        Ok(Action::Nop)
    }

    fn key_direction(key: KeyEvent) -> Option<Direction> {
        match key {
            KeyEvent {
                main_key: Key::Up, alt: false, shift: false, ..
            } => Some(Direction::Up),

            KeyEvent {
                main_key: Key::Down, alt: false, shift: false, ..
            } => Some(Direction::Down),

            KeyEvent {
                main_key: Key::Left, alt: false, shift: false, ..
            } => Some(Direction::Left),

            KeyEvent {
                main_key: Key::Right, alt: false, shift: false, ..
            } => Some(Direction::Right),

            _ => None,
        }
    }

    async fn dispatch_resize<'term>(
        &mut self,
        evt: ResizeEvent,
        screen: &mut Screen<'term>,
    ) -> Result<Action> {
        if let Some(size) = evt.size {
            self.resize_camera(size);
            self.render(screen).await?;
            Ok(Action::Nop)
        } else {
            Ok(Action::Pause)
        }
    }

    async fn dispatch_no_event<'term>(
        &mut self,
        _screen: &mut Screen<'term>,
    ) -> Result<Action> {
        Ok(Action::Nop)
    }

    /// Renders debug_stats and everything on the camera.
    async fn render<'guard>(&self, screen: &mut Screen<'guard>) -> Result<()> {
        screen.clear(BasicColor::Black.into());
        self.render_map(screen).await?;
        if self.settings.debug {
            self.render_debug_stats(screen).await?;
        }
        self.render_stats(screen).await?;
        Ok(())
    }

    /// Renders map in most of the screen.
    async fn render_map<'guard>(
        &self,
        screen: &mut Screen<'guard>,
    ) -> Result<()> {
        let rect = self.camera.rect();
        let mut entities = HashSet::new();

        for point in rect.rows() {
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
        screen: &mut Screen<'guard>,
    ) -> Result<()> {
        let text = format!(
            "♡ {:02}/{:02}",
            self.player.health(),
            self.player.max_health()
        );
        screen.styled_text(
            &tstring![text],
            Style::default().top_margin(screen.size().y - 2),
        );
        screen.styled_text(
            &self.message,
            Style::default().top_margin(screen.size().y - 1),
        );
        Ok(())
    }

    /// Renders statistics in the bottom of the screen.
    async fn render_debug_stats<'guard>(
        &self,
        screen: &mut Screen<'guard>,
    ) -> Result<()> {
        let pos = self.player.head().center_origin();
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
        screen.styled_text(&tstring![string], Style::default().align(1, 2));
        Ok(())
    }

    /// Updates the camera acording to the available size.
    fn resize_camera(&mut self, mut screen_size: Vec2<Coord>) {
        screen_size.y -= self.debug_stats_height();
        screen_size.y -= self.stats_height();
        self.camera = Camera::new(
            self.player.head(),
            screen_size,
            Vec2 { x: 0, y: self.debug_stats_height() },
        );
    }

    fn stats_height(&self) -> Coord {
        2
    }

    fn debug_stats_height(&self) -> Coord {
        if self.settings.debug {
            1
        } else {
            0
        }
    }
}

/// Coordinates of where the game Camera is showing.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    /// Crop of the screen that the player sees.
    rect: Rect<Coord>,
    offset: Vec2<Coord>,
}

impl Camera {
    /// Builds a new Camera from a position approximately in the center and the
    /// available size.
    pub fn new(
        center: Vec2<Coord>,
        screen_size: Vec2<Coord>,
        offset: Vec2<Coord>,
    ) -> Self {
        Self {
            rect: Rect {
                start: center.zip_with(screen_size, |center, screen_size| {
                    center.saturating_sub(screen_size / 2)
                }),
                size: screen_size,
            },
            offset,
        }
    }

    #[inline]
    /// Returns the crop of this camera.
    pub fn rect(self) -> Rect<Coord> {
        self.rect
    }

    #[inline]
    /// Returns the screen offset of this camera.
    pub fn offset(self) -> Vec2<Coord> {
        self.offset
    }

    /// Updates the camera to follow the center of the player with at least the
    /// given distance from the center to the edges.
    pub fn update(
        &mut self,
        direc: Direction,
        center: Vec2<Coord>,
        threshold: Ratio<Coord>,
    ) -> bool {
        let dist = (Ratio::from(self.rect.size[direc.axis()]) * threshold)
            .to_integer();
        match direc {
            Direction::Up => {
                let diff = center.y.checked_sub(self.rect.start.y);
                if diff.filter(|&y| y >= dist).is_none() {
                    self.rect.start.y = center.y.saturating_sub(dist);
                    true
                } else {
                    false
                }
            },

            Direction::Down => {
                let diff = self.rect.end().y.checked_sub(center.y + 1);
                if diff.filter(|&y| y >= dist).is_none() {
                    self.rect.start.y =
                        (center.y - self.rect.size.y).saturating_add(dist + 1);
                    true
                } else {
                    false
                }
            },

            Direction::Left => {
                let diff = center.x.checked_sub(self.rect.start.x);
                if diff.filter(|&x| x >= dist).is_none() {
                    self.rect.start.x = center.x.saturating_sub(dist);
                    true
                } else {
                    false
                }
            },

            Direction::Right => {
                let diff = self.rect.end().x.checked_sub(center.x + 1);
                if diff.filter(|&x| x >= dist).is_none() {
                    self.rect.start.x =
                        (center.x - self.rect.size.x).saturating_add(dist + 1);
                    true
                } else {
                    false
                }
            },
        }
    }

    /// Converts an absolute point in the map to a point in the screen.
    pub fn convert(self, point: Vec2<Coord>) -> Option<Vec2<Coord>> {
        if self.rect.has_point(point) {
            Some(
                point
                    .zip_with(self.rect.start, Sub::sub)
                    .zip_with(self.offset, Add::add),
            )
        } else {
            None
        }
    }
}
