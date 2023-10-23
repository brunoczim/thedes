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
        input::InputDialog,
        menu::{Menu, MenuOption},
    },
};
use gardiz::{coord::Vec2, direc::Direction, rect::Rect};
use num::rational::Ratio;
use tokio::{
    net::{TcpStream, ToSocketAddrs},
    time,
};

use crate::{
    domain::{Coord, GameSnapshot, Ground, Map, PlayerName},
    error::Result,
    message::{
        self,
        ClientRequest,
        GetSnapshotRequest,
        LoginRequest,
        LoginResponse,
        MoveClientPlayerRequest,
        MoveClientPlayerResponse,
    },
};

const MAX_ADDRESS_SIZE: TermCoord = 45 + 2 + 1 + 5;

const MAX_NAME_SIZE: TermCoord = 9;

const MIN_SCREEN_SIZE: Vec2<TermCoord> = Vec2 { x: 80, y: 25 };

const BORDER_THRESHOLD: Ratio<Coord> = Ratio::new_raw(1, 3);

const TICK: Duration = Duration::from_millis(25);

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
                    self.rect.start.y = (center.y - self.rect.size.y)
                        .saturating_add(dist + 1)
                        .min(Map::SIZE.y - 1);
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
                    self.rect.start.x = (center.x - self.rect.size.x)
                        .saturating_add(dist + 1)
                        .min(Map::SIZE.x - 1);
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
            Self::Connect => "connect",
            Self::Exit => "exit",
        }]
    }
}

#[derive(Debug, Clone)]
enum PauseMenuOption {
    Resume,
    Exit,
}
impl MenuOption for PauseMenuOption {
    fn name(&self) -> TermString {
        tstring![match self {
            Self::Resume => "RESUME",
            Self::Exit => "EXIT TO MAIN MENU",
        }]
    }
}

pub async fn start() -> Result<()> {
    terminal::Builder::new()
        .min_screen(MIN_SCREEN_SIZE)
        .run(run_launcher)
        .await??;
    Ok(())
}

async fn run_launcher(mut terminal: Terminal) -> Result<()> {
    let mut name_input = InputDialog::new(
        tstring!["Connect to..."],
        tstring![],
        MAX_NAME_SIZE,
        |ch| {
            "0123456789".contains(ch)
                || "abcdefghijklmnopqrstuvwxyz".contains(ch)
                || "ABCDEFGHIJKLMNOPQRSTUVWXYZ".contains(ch)
                || "-$^#@:.%".contains(ch)
        },
    );
    let mut main_menu = Menu::new(
        tstring!["T H E D E S"],
        vec![MainMenuOption::Connect, MainMenuOption::Exit],
    );
    let mut connect_input = InputDialog::new(
        tstring!["Connect to..."],
        tstring![""],
        MAX_ADDRESS_SIZE,
        |_| true,
    );
    let mut pause_menu = Menu::new(
        tstring!["T H E D E S"],
        vec![PauseMenuOption::Resume, PauseMenuOption::Exit],
    );
    while let Some(term_name) =
        name_input.select_with_cancel(&mut terminal).await?
    {
        if !term_name.is_empty() {
            let player_name = term_name.to_string();
            run_main_menu(
                &mut terminal,
                &mut main_menu,
                &mut connect_input,
                &mut pause_menu,
                player_name,
            )
            .await?;
        }
    }
    Ok(())
}

async fn run_main_menu<F>(
    terminal: &mut Terminal,
    menu: &mut Menu<MainMenuOption>,
    connect_input: &mut InputDialog<F>,
    pause_menu: &mut Menu<PauseMenuOption>,
    player_name: PlayerName,
) -> Result<()>
where
    F: FnMut(char) -> bool,
{
    loop {
        let index = menu.select(terminal).await?;
        match menu.options[index] {
            MainMenuOption::Connect => {
                run_connect_ui(
                    terminal,
                    connect_input,
                    pause_menu,
                    player_name.clone(),
                )
                .await?
            },
            MainMenuOption::Exit => break,
        }
    }
    Ok(())
}

async fn run_connect_ui<F>(
    terminal: &mut Terminal,
    input: &mut InputDialog<F>,
    pause_menu: &mut Menu<PauseMenuOption>,
    player_name: PlayerName,
) -> Result<()>
where
    F: FnMut(char) -> bool,
{
    if let Some(address_str) = input.select_with_cancel(terminal).await? {
        let server_addr: SocketAddr = address_str.parse()?;
        run_game(terminal, pause_menu, server_addr, player_name).await?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EventAction {
    Pause,
    Resize(Vec2<Coord>),
    Move(Direction),
}

async fn run_game<S>(
    terminal: &mut Terminal,
    pause_menu: &mut Menu<PauseMenuOption>,
    server_addr: S,
    player_name: PlayerName,
) -> Result<()>
where
    S: ToSocketAddrs,
{
    let mut connection = TcpStream::connect(server_addr).await?;

    let login_request = LoginRequest { player_name: player_name.clone() };
    message::send(&mut connection, login_request).await?;

    let login_response: LoginResponse =
        message::receive(&mut connection).await?;
    let mut snapshot = login_response.result?;

    let mut interval = time::interval(TICK);

    let mut camera = Camera::new(
        snapshot.players[&player_name].location.head,
        terminal.lock_now().await?.screen().size(),
        Vec2 { y: 0, x: 0 },
    );
    loop {
        let maybe_action = {
            let mut term_guard = terminal.lock_now().await?;
            render(term_guard.screen(), &snapshot, &camera).await?;
            term_guard.event().and_then(|event| match event {
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
            })
        };

        if let Some(action) = maybe_action {
            match action {
                EventAction::Pause => {
                    let selected = pause_menu.select(&mut *terminal).await?;
                    match pause_menu.options[selected] {
                        PauseMenuOption::Resume => (),
                        PauseMenuOption::Exit => break Ok(()),
                    }
                },
                EventAction::Move(direction) => {
                    message::send(
                        &mut connection,
                        ClientRequest::MoveClientPlayer(
                            MoveClientPlayerRequest { direction },
                        ),
                    )
                    .await?;
                    let response: MoveClientPlayerResponse =
                        message::receive(&mut connection).await?;
                    if response.result.is_ok() {
                        camera.update(
                            direction,
                            snapshot.players[&player_name].location.head,
                            BORDER_THRESHOLD,
                        );
                    }
                },
                EventAction::Resize(new_size) => {
                    camera = Camera::new(
                        snapshot.players[&player_name].location.head,
                        new_size,
                        Vec2 { x: 0, y: 0 },
                    );
                },
            }
        }

        message::send(
            &mut connection,
            ClientRequest::GetSnapshotRequest(GetSnapshotRequest),
        )
        .await?;
        snapshot = message::receive(&mut connection).await?;
        interval.tick().await;
    }
}

async fn render(
    screen: &mut Screen<'_>,
    snapshot: &GameSnapshot,
    camera: &Camera,
) -> Result<()> {
    screen.clear(BasicColor::Black.into());
    for point in camera.rect().rows() {
        let tile = if point.x < Map::SIZE.x && point.y < Map::SIZE.y {
            Tile {
                grapheme: TermGrapheme::new_lossy(
                    if let Some(player) = snapshot.map[point].player.as_ref() {
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
                    background: match snapshot.map[point].ground {
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

        screen.set(camera.convert(point).unwrap(), tile);
    }
    Ok(())
}
