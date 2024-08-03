use std::time::Duration;

use thiserror::Error;

use crate::{
    color::{BasicColor, Color, ColorPair},
    geometry::Coord,
    CanvasError,
    TextStyle,
    Tick,
};

pub type ProgressMetric = u64;

pub trait TaskProgress {
    fn progress_goal(&self) -> ProgressMetric;
    fn current_progress(&self) -> ProgressMetric;
    fn progress_status(&self) -> String;
}

pub trait TaskReset<A> {
    type Output;
    type Error;

    fn reset(&mut self, args: A) -> Result<Self::Output, Self::Error>;
}

pub trait TaskTick<A> {
    type Output;
    type Error;

    fn on_tick(
        &mut self,
        tick: &mut Tick,
        args: A,
    ) -> Result<Option<Self::Output>, Self::Error>;
}

#[derive(Debug, Error)]
pub enum ResetError<E> {
    #[error(transparent)]
    Task(E),
}

#[derive(Debug, Error)]
pub enum TickError<E> {
    #[error(transparent)]
    Task(E),
    #[error("Failed to manipulate canvas")]
    Canvas(
        #[source]
        #[from]
        CanvasError,
    ),
}

#[derive(Debug, Clone)]
pub struct Config {
    title: String,
    title_y: Coord,
    title_colors: ColorPair,
    pad_after_title: Coord,
    bar_size: Coord,
    bar_colors: ColorPair,
    pad_after_bar: Coord,
    pad_after_perc: Coord,
    stat_colors: ColorPair,
    background: Color,
}

impl Config {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            title_y: 1,
            title_colors: ColorPair::default(),
            pad_after_title: 2,
            pad_after_bar: 0,
            pad_after_perc: 1,
            bar_size: 32,
            bar_colors: ColorPair {
                foreground: BasicColor::White.into(),
                background: BasicColor::DarkGray.into(),
            },
            stat_colors: ColorPair::default(),
            background: BasicColor::Black.into(),
        }
    }

    pub fn with_title_colors(self, colors: ColorPair) -> Self {
        Self { title_colors: colors, ..self }
    }

    pub fn with_title_y(self, y: Coord) -> Self {
        Self { title_y: y, ..self }
    }

    pub fn with_pad_after_title(self, padding: Coord) -> Self {
        Self { pad_after_title: padding, ..self }
    }

    pub fn with_bar_size(self, size: Coord) -> Self {
        Self { bar_size: size, ..self }
    }

    pub fn with_bar_colors(self, colors: ColorPair) -> Self {
        Self { bar_colors: colors, ..self }
    }

    pub fn with_pad_after_bar(self, padding: Coord) -> Self {
        Self { pad_after_bar: padding, ..self }
    }

    pub fn with_pad_after_perc(self, padding: Coord) -> Self {
        Self { pad_after_perc: padding, ..self }
    }

    pub fn with_stat_colors(self, colors: ColorPair) -> Self {
        Self { stat_colors: colors, ..self }
    }

    pub fn with_background(self, color: Color) -> Self {
        Self { background: color, ..self }
    }

    pub fn finish<T>(self, task: T) -> TaskMonitor<T> {
        TaskMonitor { config: self, task }
    }
}

#[derive(Debug, Clone)]
pub struct TaskMonitor<T> {
    config: Config,
    task: T,
}

impl<T> TaskMonitor<T> {
    pub fn reset<A>(
        &mut self,
        args: A,
    ) -> Result<T::Output, ResetError<T::Error>>
    where
        T: TaskReset<A>,
    {
        self.task.reset(args).map_err(ResetError::Task)
    }

    pub fn on_tick<A, B, E>(
        &mut self,
        tick: &mut Tick,
        args: &mut A,
    ) -> Result<Option<B>, TickError<E>>
    where
        T: for<'a> TaskTick<&'a mut A, Output = B, Error = E> + TaskProgress,
    {
        let output = loop {
            let output =
                self.task.on_tick(tick, args).map_err(TickError::Task)?;
            if output.is_some() || tick.time_available() == Duration::ZERO {
                break output;
            }
        };
        self.render(tick)?;
        Ok(output)
    }

    fn render(&self, tick: &mut Tick) -> Result<(), CanvasError>
    where
        T: TaskProgress,
    {
        tick.screen_mut().clear_canvas(self.config.background)?;
        self.render_title(tick)?;
        self.render_bar(tick)?;
        self.render_perc(tick)?;
        self.render_absolute(tick)?;
        Ok(())
    }

    fn render_bar(&self, tick: &mut Tick) -> Result<(), CanvasError>
    where
        T: TaskProgress,
    {
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.config.bar_colors)
            .with_top_margin(self.y_of_bar());
        let mut text = String::new();
        let normalized_progress = self.task.current_progress()
            * ProgressMetric::from(self.config.bar_size)
            / self.task.progress_goal();
        let normalized_progress = normalized_progress as Coord;
        for _ in 0 .. normalized_progress {
            text.push_str("█");
        }
        for _ in normalized_progress .. self.config.bar_size {
            text.push_str(" ");
        }
        tick.screen_mut().styled_text(&text, &style)?;
        Ok(())
    }

    fn render_title(&self, tick: &mut Tick) -> Result<(), CanvasError> {
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.config.title_colors)
            .with_top_margin(self.config.title_y);
        tick.screen_mut().styled_text(&self.config.title, &style)?;
        Ok(())
    }

    fn render_perc(&self, tick: &mut Tick) -> Result<(), CanvasError>
    where
        T: TaskProgress,
    {
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.config.stat_colors)
            .with_top_margin(self.y_of_perc());
        let perc =
            self.task.current_progress() * 100 / self.task.progress_goal();
        let text = format!("{perc}%");
        tick.screen_mut().styled_text(&text, &style)?;
        Ok(())
    }

    fn render_absolute(&self, tick: &mut Tick) -> Result<(), CanvasError>
    where
        T: TaskProgress,
    {
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.config.stat_colors)
            .with_top_margin(self.y_of_absolute());
        let status = self.task.current_progress();
        let goal = self.task.progress_goal();
        let text = format!("{status}/{goal}");
        tick.screen_mut().styled_text(&text, &style)?;
        Ok(())
    }

    fn y_of_bar(&self) -> Coord {
        self.config.pad_after_title + 1 + self.config.title_y
    }

    fn y_of_perc(&self) -> Coord {
        self.y_of_bar() + 1 + self.config.pad_after_bar
    }

    fn y_of_absolute(&self) -> Coord {
        self.y_of_perc() + 1 + self.config.pad_after_perc
    }
}
