use std::time::Duration;

use crate::{
    app::App,
    color::{BasicColor, ColorPair},
    geometry::CoordPair,
    ExecutionError,
    Tick,
};

#[derive(Debug)]
pub struct Config {
    canvas_size: CoordPair,
    default_colors: ColorPair,
    tick_interval: Duration,
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
            tick_interval: Duration::from_millis(1),
            render_ticks: 16,
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

    pub fn with_tick_interval(self, interval: Duration) -> Self {
        Self { tick_interval: interval, ..self }
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

    pub(crate) fn tick_interval(&self) -> Duration {
        self.tick_interval
    }

    pub fn run<F, E>(&self, on_tick: F) -> Result<(), ExecutionError<E>>
    where
        F: FnMut(&mut Tick) -> Result<bool, E>,
    {
        let mut app = App::new(&self, on_tick)?;
        while app.next_tick()? {}
        Ok(())
    }
}
