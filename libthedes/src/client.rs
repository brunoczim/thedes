use std::{
    net::SocketAddr,
    ops::{Add, Sub},
    time::Duration,
};

use andiskaz::{
    color::{BasicColor, Color2},
    coord::Coord as TermCoord,
    event::{Event, Key, KeyEvent},
    screen::Screen,
    string::{TermGrapheme, TermString},
    terminal::{self, Terminal},
    tile::Tile,
    tstring,
    ui::{
        info::InfoDialog,
        input::InputDialog,
        menu::{Menu, MenuOption},
    },
};
use anyhow::anyhow;
use gardiz::{coord::Vec2, direc::Direction, rect::Rect};
use num::rational::Ratio;
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{self, Interval},
};

use crate::{
    domain::{map::Ground, plane::Coord, player, state::GameSnapshot},
    error::Result,
    message::{
        self,
        ClientRequest,
        GetPlayerRequest,
        GetPlayerResponse,
        GetSnapshotRequest,
        GetSnapshotResponse,
        LoginRequest,
        LoginResponse,
        LogoutRequest,
        LogoutResponse,
        MoveClientPlayerRequest,
        MoveClientPlayerResponse,
    },
};

const MAX_ADDRESS_SIZE: TermCoord = 45 + 2 + 1 + 5;

const MAX_NAME_SIZE: TermCoord = player::Name::MAX_LEN as u16;

const MIN_SCREEN_SIZE: Vec2<TermCoord> = Vec2 { x: 80, y: 25 };

const BORDER_THRESHOLD: Ratio<Coord> = Ratio::new_raw(1, 3);

const TICK: Duration = Duration::from_millis(50);

pub async fn run() -> Result<()> {
    terminal::Builder::new()
        .min_screen(MIN_SCREEN_SIZE)
        .run(|terminal| async move { Ui::new().run_launcher(terminal).await })
        .await??;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Camera {
    rect: Rect<Coord>,
    offset: Vec2<Coord>,
}

impl Camera {
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

    pub fn rect(self) -> Rect<Coord> {
        self.rect
    }

    pub fn update(
        &mut self,
        direction: Direction,
        center: Vec2<Coord>,
        threshold: Ratio<Coord>,
    ) -> bool {
        let dist = (Ratio::from(self.rect.size[direction.axis()]) * threshold)
            .to_integer();
        match direction {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainMenuOption {
    Connect,
    Exit,
}

impl MenuOption for MainMenuOption {
    fn name(&self) -> TermString {
        tstring![match self {
            Self::Connect => "CONNECT",
            Self::Exit => "RETURN TO LAUNCHER",
        }]
    }
}

#[derive(Debug, Clone)]
enum PauseMenuOption {
    Resume,
    QuitGame,
}

impl MenuOption for PauseMenuOption {
    fn name(&self) -> TermString {
        tstring![match self {
            Self::Resume => "RESUME",
            Self::QuitGame => "QUIT GAME",
        }]
    }
}

#[derive(Debug, Clone)]
struct Ui {
    launcher_input: InputDialog<fn(char) -> bool>,
    main_menu: Menu<MainMenuOption>,
    connect_input: InputDialog<fn(char) -> bool>,
    pause_menu: Menu<PauseMenuOption>,
}

impl Ui {
    pub fn new() -> Self {
        let mut this = Self {
            launcher_input: InputDialog::new(
                tstring!["Connect to..."],
                tstring![],
                MAX_NAME_SIZE,
                |ch| ch == '-' || ch.is_ascii_alphanumeric() || ch == '_',
            ),
            main_menu: Menu::new(
                tstring!["T H E D E S"],
                vec![MainMenuOption::Connect, MainMenuOption::Exit],
            ),
            connect_input: InputDialog::new(
                tstring!["Connect to..."],
                tstring![""],
                MAX_ADDRESS_SIZE,
                |_| true,
            ),
            pause_menu: Menu::new(
                tstring!["T H E D E S"],
                vec![PauseMenuOption::Resume, PauseMenuOption::QuitGame],
            ),
        };

        this.launcher_input.ok_label = tstring!["OK"];
        this.launcher_input.cancel_label = tstring!["EXIT"];

        this
    }

    pub async fn run_launcher(&mut self, mut terminal: Terminal) -> Result<()> {
        while let Some(term_name) =
            self.launcher_input.select_with_cancel(&mut terminal).await?
        {
            if !term_name.is_empty() {
                let player_name = term_name.as_str().parse()?;
                self.run_main_menu(&mut terminal, player_name).await?;
            }
        }
        Ok(())
    }

    async fn run_main_menu(
        &mut self,
        terminal: &mut Terminal,
        player_name: player::Name,
    ) -> Result<()> {
        loop {
            let index = self.main_menu.select(terminal).await?;
            match self.main_menu.options[index] {
                MainMenuOption::Connect => {
                    self.run_connect(terminal, player_name).await?
                },
                MainMenuOption::Exit => break,
            }
        }
        Ok(())
    }

    async fn run_connect(
        &mut self,
        terminal: &mut Terminal,
        player_name: player::Name,
    ) -> Result<()> {
        if let Some(address_str) =
            self.connect_input.select_with_cancel(terminal).await?
        {
            let try_connect = async {
                let server_addr = address_str.parse()?;
                self.run_game(terminal, server_addr, player_name).await?;
                Result::<_>::Ok(())
            };
            if let Err(error) = try_connect.await {
                tracing::warn!(
                    "Connection failed: {}\nBacktrace: {}",
                    error,
                    error.backtrace(),
                );
                let dialog = InfoDialog::new(
                    tstring!["Connection Failed"],
                    tstring![error.to_string()],
                );
                dialog.run(terminal).await?;
            }
        }
        Ok(())
    }

    async fn run_game(
        &mut self,
        terminal: &mut Terminal,
        server_addr: SocketAddr,
        player_name: player::Name,
    ) -> Result<()> {
        let session =
            Session::connect(self, terminal, server_addr, player_name).await?;
        session.run(terminal).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EventAction {
    Pause,
    Resize(Vec2<Coord>),
    Move(Direction),
}

#[derive(Debug)]
struct Session<'ui> {
    ui: &'ui mut Ui,
    connection: TcpStream,
    player_name: player::Name,
    camera: Camera,
    snapshot: GameSnapshot,
    interval: Interval,
    running: bool,
}

impl<'ui> Session<'ui> {
    pub async fn connect(
        ui: &'ui mut Ui,
        terminal: &mut Terminal,
        server_addr: SocketAddr,
        player_name: player::Name,
    ) -> Result<Session<'ui>> {
        tracing::debug!("Connecting to server {}", server_addr);

        let mut connection = TcpStream::connect(server_addr).await?;

        let login_request = LoginRequest { player_name };
        message::send(&mut connection, login_request).await?;

        let login_response: LoginResponse =
            message::receive(&mut connection).await?;
        login_response.result?;

        let interval = time::interval(TICK);

        let get_player_request =
            ClientRequest::GetPlayerRequest(GetPlayerRequest { player_name });
        message::send(&mut connection, get_player_request).await?;

        let get_player_response: GetPlayerResponse =
            message::receive(&mut connection).await?;
        let player = get_player_response.result?;

        let camera = Camera::new(
            player.location.head,
            terminal.lock_now().await?.screen().size(),
            Vec2 { y: 0, x: 0 },
        );

        let get_snapshot_request =
            ClientRequest::GetSnapshotRequest(GetSnapshotRequest {
                view: camera.rect(),
            });
        message::send(&mut connection, get_snapshot_request).await?;

        let get_snapshot_response: GetSnapshotResponse =
            message::receive(&mut connection).await?;
        let snapshot = get_snapshot_response.result?;

        Ok(Self {
            ui,
            connection,
            player_name,
            camera,
            snapshot,
            interval,
            running: true,
        })
    }

    pub async fn run(mut self, terminal: &mut Terminal) -> Result<()> {
        let run_result = self.do_run(terminal).await;
        tracing::debug!("Disconnecting...");
        let cleanup_result = self.cleanup().await;
        run_result?;
        cleanup_result
    }

    async fn do_run(&mut self, terminal: &mut Terminal) -> Result<()> {
        while self.running {
            self.tick(terminal).await?;
        }
        Ok(())
    }

    async fn cleanup(mut self) -> Result<()> {
        tracing::debug!("Cleaning up connection");
        message::send(
            &mut self.connection,
            ClientRequest::LogoutRequest(LogoutRequest),
        )
        .await?;
        let response: LogoutResponse =
            message::receive(&mut self.connection).await?;
        response.result?;
        self.connection.shutdown().await?;
        Ok(())
    }

    async fn tick(&mut self, terminal: &mut Terminal) -> Result<()> {
        let maybe_action = {
            let mut term_guard = terminal.lock_now().await?;
            self.render(term_guard.screen()).await?;
            term_guard.event().and_then(|event| self.event_action(event))
        };

        if let Some(action) = maybe_action {
            self.run_event_action(terminal, action).await?;
        }

        if self.running {
            self.next().await?;
        }

        Ok(())
    }

    async fn next(&mut self) -> Result<()> {
        let get_snapshot_request =
            ClientRequest::GetSnapshotRequest(GetSnapshotRequest {
                view: self.camera.rect(),
            });
        message::send(&mut self.connection, get_snapshot_request).await?;
        let get_snapshot_response: GetSnapshotResponse =
            message::receive(&mut self.connection).await?;
        self.snapshot = get_snapshot_response.result?;
        self.interval.tick().await;
        Ok(())
    }

    fn event_action(&mut self, event: Event) -> Option<EventAction> {
        match event {
            Event::Resize(resize_event) => {
                resize_event.size.map(EventAction::Resize)
            },
            Event::Key(KeyEvent { main_key: Key::Esc, .. }) => {
                Some(EventAction::Pause)
            },
            Event::Key(key_event)
                if key_event.ctrl == false
                    && key_event.alt == false
                    && key_event.shift == false =>
            {
                match key_event.main_key {
                    Key::Up => Some(EventAction::Move(Direction::Up)),
                    Key::Down => Some(EventAction::Move(Direction::Down)),
                    Key::Left => Some(EventAction::Move(Direction::Left)),
                    Key::Right => Some(EventAction::Move(Direction::Right)),
                    _ => None,
                }
            },

            _ => None,
        }
    }

    async fn run_event_action(
        &mut self,
        terminal: &mut Terminal,
        action: EventAction,
    ) -> Result<()> {
        match action {
            EventAction::Pause => {
                let selected =
                    self.ui.pause_menu.select(&mut *terminal).await?;
                match self.ui.pause_menu.options[selected] {
                    PauseMenuOption::Resume => (),
                    PauseMenuOption::QuitGame => self.running = false,
                }
            },

            EventAction::Move(direction) => {
                message::send(
                    &mut self.connection,
                    ClientRequest::MoveClientPlayer(MoveClientPlayerRequest {
                        direction,
                    }),
                )
                .await?;
                let response: MoveClientPlayerResponse =
                    message::receive(&mut self.connection).await?;
                if response.result.is_ok() {
                    self.camera.update(
                        direction,
                        self.snapshot.players[&self.player_name].location.head,
                        BORDER_THRESHOLD,
                    );
                }
            },

            EventAction::Resize(new_size) => {
                self.camera = Camera::new(
                    self.snapshot.players[&self.player_name].location.head,
                    new_size,
                    Vec2 { x: 0, y: 0 },
                );
            },
        }

        Ok(())
    }

    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all))]
    async fn render(&mut self, screen: &mut Screen<'_>) -> Result<()> {
        screen.clear(BasicColor::Black.into());
        for point in self.camera.rect().rows() {
            let tile = self.tile_at(point)?;
            screen.set(self.camera.convert(point).unwrap(), tile);
        }
        Ok(())
    }

    fn tile_at(&mut self, point: Vec2<Coord>) -> Result<Tile> {
        let tile = if point.y <= self.snapshot.map.view().end_inclusive().y
            && point.x <= self.snapshot.map.view().end_inclusive().x
        {
            Tile {
                grapheme: TermGrapheme::new_lossy(
                    if let Some(player_name) =
                        self.snapshot.map[point].player.into_option()
                    {
                        let player = self
                            .snapshot
                            .players
                            .get(&player_name)
                            .ok_or_else(|| {
                                anyhow!(
                                    "inconsistent server response: player {} \
                                     does not exist",
                                    player_name
                                )
                            })?;
                        if player.location.head == point {
                            "O"
                        } else {
                            match player.location.facing {
                                Direction::Up => "∧",
                                Direction::Down => "∨",
                                Direction::Left => "<",
                                Direction::Right => ">",
                            }
                        }
                    } else {
                        " "
                    },
                ),
                colors: Color2 {
                    foreground: BasicColor::DarkGray.into(),
                    background: match self.snapshot.map[point].ground {
                        Ground::Grass => BasicColor::LightGreen.into(),
                        Ground::Sand => BasicColor::LightYellow.into(),
                    },
                },
            }
        } else {
            Tile {
                grapheme: TermGrapheme::new_lossy(" "),
                colors: Color2 {
                    foreground: BasicColor::White.into(),
                    background: BasicColor::Black.into(),
                },
            }
        };

        Ok(tile)
    }
}
