use std::fmt;

use num::rational::Ratio;
use thedes_domain::game::Game;
use thedes_geometry::orientation::Direction;
use thedes_session::{EventError, Session};
use thedes_tui::{
    core::{
        App,
        event::{Event, Key, KeyEvent},
        input,
        screen::FlushError,
    },
    menu::{self, Menu},
};
use thiserror::Error;

pub fn default_key_bindings() -> KeyBindingMap {
    let mut map = KeyBindingMap::new()
        .with(Key::Esc, Command::Pause)
        .with(Key::Char('q'), Command::Pause)
        .with(Key::Char('Q'), Command::Pause);

    let arrow_key_table = [
        (Key::Up, Direction::Up),
        (Key::Left, Direction::Left),
        (Key::Down, Direction::Down),
        (Key::Right, Direction::Right),
    ];
    for (key, direction) in arrow_key_table {
        let pointer_key = key;
        let head_key =
            KeyEvent { main_key: key, ctrl: true, alt: false, shift: false };
        map = map
            .with(pointer_key, ControlCommand::MovePlayerPointer(direction))
            .with(head_key, ControlCommand::MovePlayerHead(direction));
    }

    map
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to build pause menu")]
    PauseMenu(
        #[source]
        #[from]
        menu::Error,
    ),
    #[error("Pause menu is inconsistent, quit not found")]
    MissingPauseQuit,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("TUI cancelled")]
    Cancelled,
    #[error("Input driver was cancelled")]
    InputCancelled(
        #[source]
        #[from]
        input::ReadError,
    ),
    #[error("Failed to render session")]
    Render(
        #[source]
        #[from]
        thedes_session::RenderError,
    ),
    #[error("Failed to move player around")]
    MoveAround(
        #[source]
        #[from]
        thedes_session::MoveAroundError,
    ),
    #[error("Failed to move player with quick step")]
    QuickStep(
        #[source]
        #[from]
        thedes_session::QuickStepError,
    ),
    #[error("Pause menu failed to run")]
    PauseMenu(
        #[source]
        #[from]
        menu::Error,
    ),
    #[error("Failed to flush commands to screen")]
    Flush(
        #[from]
        #[source]
        FlushError,
    ),
    #[error("Failed to simulate events")]
    Event(
        #[from]
        #[source]
        EventError,
    ),
}

pub type KeyBindingMap = thedes_tui::key_bindings::KeyBindingMap<Command>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Command {
    Pause,
    Control(ControlCommand),
}

impl From<ControlCommand> for Command {
    fn from(command: ControlCommand) -> Self {
        Command::Control(command)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ControlCommand {
    MovePlayerHead(Direction),
    MovePlayerPointer(Direction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PauseMenuItem {
    Continue,
    Quit,
}

impl fmt::Display for PauseMenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Continue => "Continue Game",
            Self::Quit => "Quit Game",
        })
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    control_events_per_tick: Ratio<u32>,
    inner: thedes_session::Config,
    key_bindings: KeyBindingMap,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            control_events_per_tick: Ratio::new(1, 8),
            inner: thedes_session::Config::new(),
            key_bindings: default_key_bindings(),
        }
    }

    pub fn with_control_events_per_tick(
        self,
        events: impl Into<Ratio<u32>>,
    ) -> Self {
        Self { control_events_per_tick: events.into(), ..self }
    }

    pub fn with_key_bindings(self, key_bindings: KeyBindingMap) -> Self {
        Self { key_bindings, ..self }
    }

    pub fn with_inner(self, config: thedes_session::Config) -> Self {
        Self { inner: config, ..self }
    }

    pub fn finish(self, game: Game) -> Result<Component, InitError> {
        let pause_menu_items = [PauseMenuItem::Continue, PauseMenuItem::Quit];

        let quit_position = pause_menu_items
            .iter()
            .position(|item| *item == PauseMenuItem::Quit)
            .ok_or(InitError::MissingPauseQuit)?;

        let pause_menu_bindings = menu::default_key_bindings()
            .with(Key::Char('q'), menu::Command::SelectConfirm(quit_position));

        let pause_menu = Menu::new("!! Game is paused !!", pause_menu_items)?
            .with_keybindings(pause_menu_bindings);

        Ok(Component {
            inner: self.inner.finish(game),
            control_events_per_tick: self.control_events_per_tick,
            controls_left: Ratio::new(0, 1),
            key_bindings: self.key_bindings,
            pause_menu,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Component {
    inner: Session,
    control_events_per_tick: Ratio<u32>,
    controls_left: Ratio<u32>,
    key_bindings: KeyBindingMap,
    pause_menu: Menu<PauseMenuItem>,
}

impl Component {
    pub async fn run(&mut self, app: &mut App) -> Result<(), Error> {
        while self.handle_input(app).await? {
            let more_controls_left =
                self.controls_left + self.control_events_per_tick;
            if more_controls_left < self.control_events_per_tick.ceil() * 2 {
                self.controls_left = more_controls_left;
            }
            self.inner.tick_event()?;
            self.inner.render(app)?;
            app.canvas.flush()?;

            tokio::select! {
                _ = app.tick_session.tick() => (),
                _ = app.cancel_token.cancelled() => Err(Error::Cancelled)?,
            }
        }
        Ok(())
    }

    async fn handle_input(&mut self, app: &mut App) -> Result<bool, Error> {
        let events: Vec<_> = app.events.read_until_now()?.collect();

        for event in events {
            match event {
                Event::Key(key) => {
                    if !self.handle_key(app, key).await? {
                        return Ok(false);
                    }
                },
                Event::Paste(_) => (),
            }
        }

        Ok(true)
    }

    async fn handle_key(
        &mut self,
        app: &mut App,
        key: KeyEvent,
    ) -> Result<bool, Error> {
        if let Some(command) = self.key_bindings.command_for(key) {
            match command {
                Command::Pause => {
                    self.pause_menu.run(app).await?;
                    match self.pause_menu.output() {
                        PauseMenuItem::Continue => (),
                        PauseMenuItem::Quit => return Ok(false),
                    }
                },
                Command::Control(command) => {
                    if self.controls_left >= Ratio::ONE {
                        self.controls_left -= Ratio::ONE;
                        self.handle_control(app, *command)?;
                    }
                },
            }
        }

        Ok(true)
    }

    fn handle_control(
        &mut self,
        app: &mut App,
        command: ControlCommand,
    ) -> Result<(), Error> {
        match command {
            ControlCommand::MovePlayerHead(direction) => {
                self.inner.quick_step(app, direction)?;
            },
            ControlCommand::MovePlayerPointer(direction) => {
                self.inner.move_around(app, direction)?;
            },
        }
        Ok(())
    }
}
