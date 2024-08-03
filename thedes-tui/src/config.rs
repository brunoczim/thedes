use std::time::Duration;

use thiserror::Error;

use crate::{
    color::{BasicColor, ColorPair},
    geometry::CoordPair,
    runtime::Runtime,
    ExecutionError,
    Tick,
};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(
        "Tick interval {:?} must be greater than minimum poll interval {:?}",
        .tick_interval,
        .min_poll_interval,
    )]
    IntervalBounds { tick_interval: Duration, min_poll_interval: Duration },
}

#[derive(Debug)]
pub struct Config {
    canvas_size: CoordPair,
    default_colors: ColorPair,
    tick_interval: Duration,
    min_poll_interval: Duration,
    render_ticks: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            canvas_size: CoordPair { x: 78, y: 22 },
            default_colors: ColorPair {
                foreground: BasicColor::LightGray.into(),
                background: BasicColor::Black.into(),
            },
            tick_interval: Duration::from_millis(8),
            min_poll_interval: Duration::from_micros(10),
            render_ticks: 2,
        }
    }
}

impl Config {
    pub fn with_canvas_size(self, size: CoordPair) -> Self {
        Self { canvas_size: size, ..self }
    }

    pub fn with_default_colors(self, colors: ColorPair) -> Self {
        Self { default_colors: colors, ..self }
    }

    pub fn with_tick_interval(
        self,
        interval: Duration,
    ) -> Result<Self, ConfigError> {
        if interval <= self.min_poll_interval {
            Err(ConfigError::IntervalBounds {
                tick_interval: interval,
                min_poll_interval: self.min_poll_interval,
            })?
        }
        Ok(Self { tick_interval: interval, ..self })
    }

    pub fn with_min_poll_interval(
        self,
        interval: Duration,
    ) -> Result<Self, ConfigError> {
        if self.min_poll_interval <= interval {
            Err(ConfigError::IntervalBounds {
                tick_interval: self.tick_interval,
                min_poll_interval: interval,
            })?
        }
        Ok(Self { tick_interval: interval, ..self })
    }

    pub fn with_render_ticks(self, ticks: u16) -> Self {
        Self { render_ticks: ticks, ..self }
    }

    pub(crate) fn canvas_size(&self) -> CoordPair {
        self.canvas_size
    }

    pub(crate) fn default_colors(&self) -> ColorPair {
        self.default_colors
    }

    pub(crate) fn render_ticks(&self) -> u16 {
        self.render_ticks
    }

    pub fn tick_interval(&self) -> Duration {
        self.tick_interval
    }

    pub fn min_poll_interval(&self) -> Duration {
        self.min_poll_interval
    }

    pub fn run<F, E>(&self, on_tick: F) -> Result<(), ExecutionError<E>>
    where
        F: FnMut(&mut Tick) -> Result<bool, E>,
    {
        let mut app = Runtime::new(&self, on_tick)?;
        while app.next_tick()? {}
        Ok(())
    }
}
