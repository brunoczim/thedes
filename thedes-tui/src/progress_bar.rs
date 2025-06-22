pub use style::Style;

mod style;

use std::panic;

use crate::{
    core::{
        App,
        event::{Event, Key, KeyEvent},
        input,
        screen::FlushError,
    },
    text,
};
use thedes_async_util::progress::{self, Progress};
use thedes_tui_core::{geometry::Coord, mutation::Set, screen};
use thiserror::Error;
use tokio::task;

pub fn default_key_bindings() -> KeyBindingMap {
    let map = KeyBindingMap::new()
        .with(Key::Esc, Command::Cancel)
        .with(Key::Char('q'), Command::Cancel)
        .with(Key::Char('Q'), Command::Cancel);
    map
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Generator task was cancelled")]
    Join(
        #[source]
        #[from]
        task::JoinError,
    ),
    #[error("Failed to flush commands to screen")]
    Flush(
        #[from]
        #[source]
        FlushError,
    ),
    #[error("Failed to render text")]
    Error(
        #[from]
        #[source]
        text::Error,
    ),
    #[error("TUI cancelled")]
    Cancelled,
    #[error("Input driver was cancelled")]
    InputCancelled(
        #[source]
        #[from]
        input::ReadError,
    ),
}

#[derive(Debug, Clone)]
pub enum Command {
    Cancel,
}

pub type KeyBindingMap = crate::key_bindings::KeyBindingMap<Command>;

#[derive(Debug, Clone)]
pub struct Component {
    key_bindings: KeyBindingMap,
    title: String,
    style: Style,
}

impl Component {
    pub fn new(title: impl AsRef<str>) -> Self {
        Self {
            title: title.as_ref().to_owned(),
            style: Style::default(),
            key_bindings: default_key_bindings(),
        }
    }

    pub async fn run<A>(
        &self,
        app: &mut App,
        monitor: progress::Monitor,
        task: A,
    ) -> Result<Option<A::Output>, Error>
    where
        A: Future + Send + 'static,
        A::Output: Send + 'static,
    {
        let cancel_token = app.cancel_token.child_token();

        let task = task::spawn({
            let cancel_token = cancel_token.clone();
            async move {
                tokio::select! {
                    output = task => Some(output),
                    _ = cancel_token.cancelled() => None,
                }
            }
        });

        let should_continue = loop {
            if !self.handle_input(app)? {
                cancel_token.cancel();
                break false;
            }
            let progress = monitor.read();
            if progress.current() >= monitor.goal() {
                break true;
            }
            self.render(app, &progress, monitor.goal())?;
            app.canvas.flush()?;
            tokio::select! {
                _ = app.tick_session.tick() => (),
                _ = app.cancel_token.cancelled() => Err(Error::Cancelled)?,
            }
        };

        let item = match task.await {
            Ok(item) => item,
            Err(join_error) => match join_error.try_into_panic() {
                Ok(payload) => panic::resume_unwind(payload),
                Err(join_error) => Err(join_error)?,
            },
        };

        Ok(item.filter(|_| should_continue))
    }

    fn handle_input(&self, app: &mut App) -> Result<bool, Error> {
        let events: Vec<_> = app.events.read_until_now()?.collect();

        for event in events {
            match event {
                Event::Key(key) => {
                    if !self.handle_key(key)? {
                        return Ok(false);
                    }
                },
                Event::Paste(_) => (),
            }
        }

        Ok(true)
    }

    fn handle_key(&self, key: KeyEvent) -> Result<bool, Error> {
        if let Some(command) = self.key_bindings.command_for(key) {
            match command {
                Command::Cancel => return Ok(false),
            }
        }

        Ok(true)
    }

    fn render(
        &self,
        app: &mut App,
        progress: &Progress,
        goal: usize,
    ) -> Result<(), Error> {
        app.canvas.queue([screen::Command::new_clear_screen(
            self.style.background(),
        )]);
        self.render_title(app)?;
        self.render_bar(app, progress, goal)?;
        self.render_perc(app, progress, goal)?;
        self.render_absolute(app, progress, goal)?;
        self.render_status(app, progress)?;
        Ok(())
    }

    fn render_title(&self, app: &mut App) -> Result<(), Error> {
        let style = text::Style::default()
            .with_align(1, 2)
            .with_colors(Set(self.style.title_colors()))
            .with_top_margin(self.style.title_y());
        text::styled(app, &self.title, &style)?;
        Ok(())
    }

    fn render_bar(
        &self,
        app: &mut App,
        progress: &Progress,
        goal: usize,
    ) -> Result<(), Error> {
        let style = text::Style::default()
            .with_align(1, 2)
            .with_colors(Set(self.style.bar_colors()))
            .with_top_margin(self.y_of_bar());
        let mut text = String::new();
        let bar_size = usize::from(self.style.bar_size());
        let normalized_progress = progress.current() * bar_size / goal;
        let normalized_progress = normalized_progress as Coord;
        for _ in 0 .. normalized_progress {
            text.push_str("█");
        }
        for _ in normalized_progress .. self.style.bar_size() {
            text.push_str(" ");
        }
        text::styled(app, &text, &style)?;
        Ok(())
    }

    fn render_perc(
        &self,
        app: &mut App,
        progress: &Progress,
        goal: usize,
    ) -> Result<(), Error> {
        let style = text::Style::default()
            .with_align(1, 2)
            .with_colors(Set(self.style.perc_colors()))
            .with_top_margin(self.y_of_perc());
        let perc = progress.current() * 100 / goal;
        let text = format!("{perc}%");
        text::styled(app, &text, &style)?;
        Ok(())
    }

    fn render_absolute(
        &self,
        app: &mut App,
        progress: &Progress,
        goal: usize,
    ) -> Result<(), Error> {
        let style = text::Style::default()
            .with_align(1, 2)
            .with_colors(Set(self.style.absolute_colors()))
            .with_top_margin(self.y_of_absolute());
        let text = format!("{current}/{goal}", current = progress.current());
        text::styled(app, &text, &style)?;
        Ok(())
    }

    fn render_status(
        &self,
        app: &mut App,
        progress: &Progress,
    ) -> Result<(), Error> {
        let style = text::Style::default()
            .with_align(1, 2)
            .with_colors(Set(self.style.status_colors()))
            .with_top_margin(self.y_of_status());
        text::styled(app, progress.status(), &style)?;
        Ok(())
    }

    fn y_of_bar(&self) -> Coord {
        self.style.pad_after_title() + 1 + self.style.title_y()
    }

    fn y_of_perc(&self) -> Coord {
        self.y_of_bar() + 1 + self.style.pad_after_bar()
    }

    fn y_of_absolute(&self) -> Coord {
        self.y_of_perc() + 1 + self.style.pad_after_perc()
    }

    fn y_of_status(&self) -> Coord {
        self.y_of_absolute() + 1 + self.style.pad_after_abs()
    }
}
