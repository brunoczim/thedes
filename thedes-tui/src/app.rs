use std::{
    collections::VecDeque,
    io,
    time::{Duration, Instant},
};

use crossterm::terminal;
use thiserror::Error;

use crate::{
    event::{Event, InternalEvent},
    geometry::CoordPair,
    screen::{RenderError, Screen},
    Config,
};

#[derive(Debug, Error)]
pub enum InitError {
    #[error("TUI screen resources failed to initialize")]
    ScreenInit(#[source] RenderError),
    #[error("TUI screen failed to be changed")]
    Enter(#[source] RenderError),
    #[error("TUI raw mode enablement failed")]
    RawMode(#[source] io::Error),
    #[error("Could not get TUI screen size")]
    FetchSize(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum ExecutionError<E> {
    #[error(transparent)]
    Init(#[from] InitError),
    #[error(transparent)]
    TickHook(E),
    #[error(transparent)]
    RenderError(#[from] RenderError),
    #[error("failed to poll event")]
    EventPoll(#[source] io::Error),
}

#[derive(Debug)]
pub struct Tick<'a> {
    stop_requested: bool,
    screen: &'a mut Screen,
    event_queue: &'a mut VecDeque<Event>,
}

impl<'a> Tick<'a> {
    pub fn request_stop(&mut self) {
        self.stop_requested = true;
    }

    pub fn screen(&self) -> &Screen {
        &*self.screen
    }

    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut *self.screen
    }

    pub fn next_event(&mut self) -> Option<Event> {
        self.event_queue.pop_front()
    }
}

#[derive(Debug)]
pub struct App<F> {
    event_queue: VecDeque<Event>,
    screen: Screen,
    render_ticks: u16,
    render_ticks_left: u16,
    then: Instant,
    tick_interval: Duration,
    corrected_interval: Duration,
    on_tick: F,
}

impl<F, E> App<F>
where
    F: FnMut(&mut Tick) -> Result<(), E>,
{
    pub fn new(config: &Config, on_tick: F) -> Result<Self, InitError> {
        terminal::enable_raw_mode().map_err(InitError::RawMode)?;

        let (x, y) = terminal::size().map_err(InitError::FetchSize)?;
        let term_size = CoordPair { x, y };

        let this = Self {
            event_queue: VecDeque::new(),
            screen: Screen::new(config, term_size)
                .map_err(InitError::ScreenInit)?,
            render_ticks: config.render_ticks(),
            render_ticks_left: 0,
            then: Instant::now(),
            tick_interval: config.tick_interval(),
            corrected_interval: config.tick_interval(),
            on_tick,
        };

        Ok(this)
    }

    pub fn next_tick(&mut self) -> Result<bool, ExecutionError<E>> {
        let mut tick = Tick {
            screen: &mut self.screen,
            event_queue: &mut self.event_queue,
            stop_requested: false,
        };
        (self.on_tick)(&mut tick).map_err(ExecutionError::TickHook)?;
        if self.render_ticks_left == 0 {
            tick.screen_mut().render()?;
            self.render_ticks_left = self.render_ticks;
        } else {
            self.render_ticks_left -= 1;
        }
        if tick.stop_requested {
            Ok(false)
        } else {
            self.collect_events()?;
            let now = Instant::now();
            self.corrected_interval = now - self.then + self.tick_interval;
            self.then = now;
            Ok(true)
        }
    }

    fn collect_events(&mut self) -> Result<(), ExecutionError<E>> {
        let mut new_term_size = None;
        let then = Instant::now();

        loop {
            let elapsed = then.elapsed();
            if elapsed >= self.corrected_interval {
                break;
            }
            if crossterm::event::poll(self.corrected_interval - elapsed)
                .map_err(ExecutionError::EventPoll)?
            {
                let crossterm_event = crossterm::event::read()
                    .map_err(ExecutionError::EventPoll)?;
                if let Some(event) =
                    InternalEvent::from_crossterm(crossterm_event)
                {
                    match event {
                        InternalEvent::External(ext_event) => {
                            self.event_queue.push_back(ext_event)
                        },
                        InternalEvent::Resize(resize_event) => {
                            new_term_size = Some(resize_event.size)
                        },
                    }
                }
            }
        }

        if let Some(size) = new_term_size {
            self.screen.term_size_changed(size)?;
        }

        Ok(())
    }
}

impl<F> Drop for App<F> {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("could not disable raw mode");
    }
}
