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
    screen::{CanvasError, Screen},
    Config,
};

#[derive(Debug, Error)]
pub enum InitError {
    #[error("TUI screen resources failed to initialize")]
    ScreenInit(#[source] CanvasError),
    #[error("TUI screen failed to be changed")]
    Enter(#[source] CanvasError),
    #[error("TUI raw mode enablement failed")]
    RawMode(#[source] io::Error),
    #[error("Could not get TUI screen size")]
    FetchSize(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum ExecutionError<E> {
    #[error(transparent)]
    Init(#[from] InitError),
    #[error("Error on application tick")]
    TickHook(#[source] E),
    #[error(transparent)]
    RenderError(#[from] CanvasError),
    #[error("Failed to poll event")]
    EventPoll(#[source] io::Error),
}

#[derive(Debug)]
pub struct Tick<'a> {
    tick_start: Instant,
    target_tick_duration: Duration,
    screen: &'a mut Screen,
    event_queue: &'a mut VecDeque<Event>,
    will_render: bool,
}

impl<'a> Tick<'a> {
    pub fn screen(&self) -> &Screen {
        &*self.screen
    }

    pub fn will_render(&self) -> bool {
        self.will_render
    }

    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut *self.screen
    }

    pub fn next_event(&mut self) -> Option<Event> {
        self.event_queue.pop_front()
    }

    pub fn time_available(&self) -> Duration {
        self.target_tick_duration.saturating_sub(self.tick_start.elapsed())
    }
}

#[derive(Debug)]
pub struct Runtime<F> {
    event_queue: VecDeque<Event>,
    screen: Screen,
    render_ticks: u16,
    render_ticks_left: u16,
    then: Instant,
    tick_interval: Duration,
    min_poll_interval: Duration,
    prev_delay: Duration,
    on_tick: F,
}

impl<F, E> Runtime<F>
where
    F: FnMut(&mut Tick) -> Result<bool, E>,
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
            min_poll_interval: config.min_poll_interval(),
            prev_delay: Duration::from_secs(0),
            on_tick,
        };

        Ok(this)
    }

    pub fn next_tick(&mut self) -> Result<bool, ExecutionError<E>> {
        let will_render = self.render_ticks_left == 0;
        let mut tick = Tick {
            tick_start: self.then,
            target_tick_duration: self.corrected_interval(),
            screen: &mut self.screen,
            event_queue: &mut self.event_queue,
            will_render,
        };
        let should_continue =
            (self.on_tick)(&mut tick).map_err(ExecutionError::TickHook)?;
        if will_render {
            tick.screen_mut().render()?;
            self.render_ticks_left = self.render_ticks;
        } else {
            self.render_ticks_left -= 1;
        }
        self.event_queue.clear();
        if should_continue {
            self.collect_events()?;
            let now = Instant::now();
            self.prev_delay =
                now - self.then + self.prev_delay - self.tick_interval;
            self.then = now;
        }
        Ok(should_continue)
    }

    fn corrected_interval(&self) -> Duration {
        self.tick_interval.saturating_sub(self.prev_delay)
    }

    fn collect_events(&mut self) -> Result<(), ExecutionError<E>> {
        let mut new_term_size = None;
        let corrected_interval = self.corrected_interval();

        let mut first = true;
        loop {
            let elapsed = self.then.elapsed();
            let was_first = first;
            first = false;
            let poll_interval = match corrected_interval.checked_sub(elapsed) {
                Some(interval) => interval,
                None if was_first => self.min_poll_interval,
                None => break,
            };
            if crossterm::event::poll(poll_interval)
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

impl<F> Drop for Runtime<F> {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("could not disable raw mode");
    }
}
