use std::time::Duration;

use thiserror::Error;

use crate::{
    color::{BasicColor, Color, ColorPair},
    event::{Event, Key, KeyEvent},
    geometry::Coord,
    CanvasError,
    TextStyle,
    Tick,
};

use super::{Cancellability, SelectionCancellability};

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
pub struct BaseConfig {
    title: String,
    title_y: Coord,
    title_colors: ColorPair,
    pad_after_title: Coord,
    bar_size: Coord,
    bar_colors: ColorPair,
    pad_after_bar: Coord,
    perc_colors: ColorPair,
    pad_after_perc: Coord,
    absolute_colors: ColorPair,
    pad_after_abs: Coord,
    status_colors: ColorPair,
    background: Color,
}

impl BaseConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            title_y: 1,
            title_colors: ColorPair::default(),
            pad_after_title: 2,
            pad_after_bar: 0,
            pad_after_perc: 1,
            pad_after_abs: 1,
            bar_size: 32,
            bar_colors: ColorPair {
                foreground: BasicColor::White.into(),
                background: BasicColor::DarkGray.into(),
            },
            absolute_colors: ColorPair::default(),
            perc_colors: ColorPair::default(),
            status_colors: ColorPair::default(),
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

    pub fn with_pad_after_abs(self, padding: Coord) -> Self {
        Self { pad_after_abs: padding, ..self }
    }

    pub fn with_absolute_colors(self, colors: ColorPair) -> Self {
        Self { absolute_colors: colors, ..self }
    }

    pub fn with_status_colors(self, colors: ColorPair) -> Self {
        Self { status_colors: colors, ..self }
    }

    pub fn with_background(self, color: Color) -> Self {
        Self { background: color, ..self }
    }
}

#[derive(Debug, Clone)]
pub struct Config<T, C> {
    pub base: BaseConfig,
    pub cancellability: C,
    pub task: T,
}

#[derive(Debug, Clone)]
pub struct TaskMonitor<T, C> {
    config: Config<T, C>,
}

impl<T, C> TaskMonitor<T, C>
where
    C: Cancellability,
{
    pub fn new(config: Config<T, C>) -> Self {
        TaskMonitor { config }
    }

    pub fn reset<A>(
        &mut self,
        args: A,
    ) -> Result<T::Output, ResetError<T::Error>>
    where
        T: TaskReset<A>,
    {
        self.config.cancellability.set_cancel_state(false);
        self.config.task.reset(args).map_err(ResetError::Task)
    }

    pub fn on_tick<A, B, E>(
        &mut self,
        tick: &mut Tick,
        args: &mut A,
    ) -> Result<Option<C::Output>, TickError<E>>
    where
        C: SelectionCancellability<B>,
        T: for<'a> TaskTick<&'a mut A, Output = B, Error = E> + TaskProgress,
    {
        while let Some(event) = tick.next_event() {
            if let Event::Key(key_evt) = event {
                if let KeyEvent {
                    main_key: Key::Esc | Key::Char('q'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                } = key_evt
                {
                    self.config.cancellability.set_cancel_state(true);
                    if let Some(cancelled) = self.config.cancellability.cancel()
                    {
                        return Ok(Some(cancelled));
                    }
                }
            }
        }

        let output = loop {
            let output = self
                .config
                .task
                .on_tick(tick, args)
                .map_err(TickError::Task)?;
            if output.is_some() || tick.time_available() == Duration::ZERO {
                break output;
            }
        };
        self.render(tick)?;
        Ok(output.map(|value| self.config.cancellability.select(value)))
    }

    fn render(&self, tick: &mut Tick) -> Result<(), CanvasError>
    where
        T: TaskProgress,
    {
        tick.screen_mut().clear_canvas(self.config.base.background)?;
        self.render_title(tick)?;
        self.render_bar(tick)?;
        self.render_perc(tick)?;
        self.render_absolute(tick)?;
        self.render_status(tick)?;
        Ok(())
    }

    fn render_bar(&self, tick: &mut Tick) -> Result<(), CanvasError>
    where
        T: TaskProgress,
    {
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.config.base.bar_colors)
            .with_top_margin(self.y_of_bar());
        let mut text = String::new();
        let current_progress = self.config.task.current_progress();
        let bar_size = ProgressMetric::from(self.config.base.bar_size);
        let goal = self.config.task.progress_goal();
        let normalized_progress = current_progress * bar_size / goal;
        let normalized_progress = normalized_progress as Coord;
        for _ in 0 .. normalized_progress {
            text.push_str("█");
        }
        for _ in normalized_progress .. self.config.base.bar_size {
            text.push_str(" ");
        }
        tick.screen_mut().styled_text(&text, &style)?;
        Ok(())
    }

    fn render_title(&self, tick: &mut Tick) -> Result<(), CanvasError> {
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.config.base.title_colors)
            .with_top_margin(self.config.base.title_y);
        tick.screen_mut().styled_text(&self.config.base.title, &style)?;
        Ok(())
    }

    fn render_perc(&self, tick: &mut Tick) -> Result<(), CanvasError>
    where
        T: TaskProgress,
    {
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.config.base.perc_colors)
            .with_top_margin(self.y_of_perc());
        let current_progress = self.config.task.current_progress();
        let goal = self.config.task.progress_goal();
        let perc = current_progress * 100 / goal;
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
            .with_colors(self.config.base.absolute_colors)
            .with_top_margin(self.y_of_absolute());
        let current = self.config.task.current_progress();
        let goal = self.config.task.progress_goal();
        let text = format!("{current}/{goal}");
        tick.screen_mut().styled_text(&text, &style)?;
        Ok(())
    }

    fn render_status(&self, tick: &mut Tick) -> Result<(), CanvasError>
    where
        T: TaskProgress,
    {
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.config.base.status_colors)
            .with_top_margin(self.y_of_status());
        let status = self.config.task.progress_status();
        tick.screen_mut().styled_text(&status, &style)?;
        Ok(())
    }

    fn y_of_bar(&self) -> Coord {
        self.config.base.pad_after_title + 1 + self.config.base.title_y
    }

    fn y_of_perc(&self) -> Coord {
        self.y_of_bar() + 1 + self.config.base.pad_after_bar
    }

    fn y_of_absolute(&self) -> Coord {
        self.y_of_perc() + 1 + self.config.base.pad_after_perc
    }

    fn y_of_status(&self) -> Coord {
        self.y_of_absolute() + 1 + self.config.base.pad_after_abs
    }
}
