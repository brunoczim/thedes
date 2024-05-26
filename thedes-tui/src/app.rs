use std::{
    io,
    thread,
    time::{Duration, Instant},
};

use crossterm::terminal;
use thiserror::Error;

use thedes_geometry::vector;

use crate::{
    geometry::Vector,
    tick::TickEvent,
    tty_screen_device::TtyScreenDevice,
};

#[derive(Debug, Error)]
pub enum AppCreationError {
    #[error("TUI application creation failed")]
    Enter(#[source] io::Error),
    #[error("TUI raw mode enablement failed")]
    RawMode(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum AppError<E> {
    #[error(transparent)]
    Creation(#[from] AppCreationError),
    #[error(transparent)]
    TickHook(E),
}

#[derive(Debug)]
pub struct App {
    screen_size: Vector,
    tty_screen: TtyScreenDevice<io::Stdout>,
}

impl App {
    pub fn new(config: Config) -> Result<Self, AppCreationError> {
        let mut this = Self {
            screen_size: config.screen_size,
            tty_screen: TtyScreenDevice::new(io::stdout()),
        };

        this.tty_screen.enter().map_err(AppCreationError::Enter)?;
        terminal::enable_raw_mode().map_err(AppCreationError::RawMode)?;

        Ok(this)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("could not disable raw mode");
        self.tty_screen.leave().expect("could not leave TUI alternate screen")
    }
}

#[derive(Debug)]
pub struct Config {
    screen_size: Vector,
    tick_interval: Duration,
    render_ticks: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            screen_size: vector![80, 24],
            tick_interval: Duration::from_millis(10),
            render_ticks: 2,
        }
    }
}

impl Config {
    pub fn with_screen_size(self, size: Vector) -> Self {
        Self { screen_size: size, ..self }
    }

    pub fn with_tick_interval(self, interval: Duration) -> Self {
        Self { tick_interval: interval, ..self }
    }

    pub fn with_render_ticks(self, ticks: u16) -> Self {
        Self { render_ticks: ticks, ..self }
    }

    pub fn run<F, E>(self, mut on_hook: F) -> Result<(), AppError<E>>
    where
        F: FnMut(&mut TickEvent) -> Result<(), E>,
    {
        let tick_interval = self.tick_interval;
        let mut app = App::new(self)?;
        let mut then = Instant::now();
        let mut interval = tick_interval;
        loop {
            let mut event = TickEvent::new(&mut app);
            on_hook(&mut event).map_err(AppError::TickHook)?;
            if event.stop_requested() {
                break;
            }
            thread::sleep(interval);
            let now = Instant::now();
            interval = now - then + tick_interval;
            then = now;
        }
        Ok(())
    }
}
