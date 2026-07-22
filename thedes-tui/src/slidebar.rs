use std::iter;

use thedes_tui_core::{
    App,
    event::{Event, Key, KeyEvent},
    geometry::Coord,
    mutation::Set,
    screen::{self, FlushError},
};

pub use style::Style;
use thiserror::Error;

use crate::text;

mod style;

pub fn default_key_bindings() -> KeyBindingMap {
    KeyBindingMap::new()
        .with(Key::Enter, Command::Back)
        .with(Key::Esc, Command::Back)
        .with(Key::Right, Command::Increase(1))
        .with(Key::Left, Command::Decrease(1))
        .with(
            KeyEvent {
                ctrl: true,
                alt: false,
                shift: false,
                main_key: Key::Right,
            },
            Command::Increase(3),
        )
        .with(
            KeyEvent {
                ctrl: true,
                alt: false,
                shift: false,
                main_key: Key::Left,
            },
            Command::Decrease(3),
        )
}

pub type KeyBindingMap = crate::key_bindings::KeyBindingMap<Command>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Back,
    Increase(Coord),
    Decrease(Coord),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("insufficient width")]
    InsufficientWidth,
    #[error("Failed to flush tiles to canvas")]
    CanvasFlush(#[from] FlushError),
    #[error("Failed to render text")]
    RenderText(#[from] text::Error),
    #[error("Menu was cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub ui_size: Coord,
    pub logical_size: Coord,
    pub logical_current: Coord,
}

#[derive(Debug, Clone)]
pub struct Slidebar {
    style: Style,
    title: String,
    message: Option<String>,
    ui_size: Coord,
    logical_size: Coord,
    ui_current: Coord,
    key_bindings: KeyBindingMap,
}

impl Slidebar {
    pub fn new(title: impl AsRef<str>, config: Config) -> Self {
        let denom = config.logical_current * (config.ui_size - 1)
            + config.logical_size / 2;
        let ui_current = denom / (config.logical_size - 1);
        Self {
            style: Style::default(),
            title: title.as_ref().to_owned(),
            ui_size: config.ui_size,
            logical_size: config.logical_size,
            ui_current,
            key_bindings: default_key_bindings(),
            message: None,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.set_title(title);
        self
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_owned();
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.set_message(message);
        self
    }

    pub fn set_message(&mut self, message: &str) -> &mut Self {
        self.message = Some(message.to_owned());
        self
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn with_style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn with_key_bindings(self, key_bindings: KeyBindingMap) -> Self {
        Self { key_bindings, ..self }
    }

    pub fn key_bindings(&self) -> &KeyBindingMap {
        &self.key_bindings
    }

    pub fn ui_size(&self) -> Coord {
        self.ui_size
    }

    pub fn logical_size(&self) -> Coord {
        self.logical_size
    }

    pub fn ui_current(&self) -> Coord {
        self.ui_current
    }

    pub fn logical_current(&self) -> Coord {
        let denom =
            self.ui_current() * (self.logical_size() - 1) + self.ui_size() / 2;
        denom / (self.ui_size() - 1)
    }

    pub fn set_ui_size(&mut self, value: Coord) {
        self.ui_size = value;
        self.set_ui_current(self.ui_current());
    }

    pub fn set_ui_current(&mut self, value: Coord) {
        self.ui_current = value.min(self.ui_size().saturating_sub(1));
    }

    pub fn run_command<F>(
        &mut self,
        app: &mut App,
        cmd: Command,
        mut on_change: F,
    ) -> Result<bool, Error>
    where
        F: FnMut(&mut App, Coord),
    {
        match cmd {
            Command::Back => return Ok(false),
            Command::Increase(amount) => {
                self.set_ui_current(self.ui_current().saturating_add(amount));
                on_change(app, self.logical_current());
            },
            Command::Decrease(amount) => {
                self.set_ui_current(self.ui_current().saturating_sub(amount));
                on_change(app, self.logical_current());
            },
        }
        Ok(true)
    }

    pub async fn run<F>(
        &mut self,
        app: &mut App,
        mut on_change: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&mut App, Coord),
    {
        self.available_width(app)?;

        while self.handle_input(app, &mut on_change)? {
            self.render(app)?;
            tokio::select! {
                _ = app.tick_session.tick() => (),
                _ = app.cancel_token.cancelled() => Err(Error::Cancelled)?,
            }
        }
        Ok(())
    }

    fn handle_input<F>(
        &mut self,
        app: &mut App,
        on_change: &mut F,
    ) -> Result<bool, Error>
    where
        F: FnMut(&mut App, Coord),
    {
        let Ok(events) = app.events.read_until_now() else {
            Err(Error::Cancelled)?
        };
        let mut events = Vec::from_iter(events).into_iter();
        let mut should_continue = true;
        while let Some(event) = events.next().filter(|_| should_continue) {
            let Event::Key(key) = event else { continue };
            let Some(&command) = self.key_bindings.command_for(key) else {
                continue;
            };
            should_continue =
                self.run_command(app, command, &mut *on_change)?;
        }
        Ok(should_continue)
    }

    fn available_width(&self, app: &App) -> Result<Coord, Error> {
        let size = app.canvas.size();
        size.y
            .checked_sub(self.style.left_margin())
            .and_then(|y| y.checked_sub(self.style.right_margin()))
            .and_then(|y| {
                Coord::try_from(
                    app.grapheme_registry.len_of(self.style.left_arrow()),
                )
                .ok()
                .and_then(|len| y.checked_sub(len))
            })
            .and_then(|y| {
                Coord::try_from(
                    app.grapheme_registry.len_of(self.style.right_arrow()),
                )
                .ok()
                .and_then(|len| y.checked_sub(len))
            })
            .ok_or(Error::InsufficientWidth)
    }

    fn render(&mut self, app: &mut App) -> Result<(), Error> {
        app.canvas
            .queue([screen::Command::ClearScreen(self.style().background())]);

        let mut height = self.style().top_margin();
        self.render_title(app, &mut height)?;

        self.render_slidebar(app, &mut height)?;
        self.render_message(app, &mut height)?;
        self.render_back(app, &mut height)?;

        app.canvas.flush()?;

        Ok(())
    }

    fn render_title(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        *height = text::styled(
            app,
            self.title(),
            &text::Style::new_with_colors(Set(self.style().title_colors()))
                .with_align(1, 2)
                .with_top_margin(*height)
                .with_left_margin(self.style().left_margin())
                .with_right_margin(self.style().right_margin()),
        )?;
        *height += self.style().title_slidebar_padding();
        Ok(())
    }

    fn render_slidebar(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        let mut bar = self.style().left_arrow().to_owned();
        let slide_handle_len =
            app.grapheme_registry.len_of(self.style().slide_handle());
        let left_count =
            usize::from(self.ui_current() + 1).saturating_sub(slide_handle_len);
        let right_count = usize::from(self.ui_size() - self.ui_current())
            .saturating_sub(slide_handle_len);
        let left_bar_ch = app.grapheme_registry.lookup(
            self.style().left_bar_ch(),
            |result| match result {
                Ok(chars) => String::from_iter(chars),
                Err(_) => String::new(),
            },
        );
        let right_bar_ch = app.grapheme_registry.lookup(
            self.style().right_bar_ch(),
            |result| match result {
                Ok(chars) => String::from_iter(chars),
                Err(_) => String::new(),
            },
        );
        bar.extend(
            iter::repeat_n(&left_bar_ch[..], left_count).flat_map(str::chars),
        );
        bar.push_str(self.style().slide_handle());
        bar.extend(
            iter::repeat_n(&right_bar_ch[..], right_count).flat_map(str::chars),
        );
        text::styled(
            app,
            &bar,
            &text::Style::new_with_colors(Set(self.style().bar_colors()))
                .with_align(1, 2)
                .with_top_margin(*height)
                .with_left_margin(self.style().left_margin())
                .with_right_margin(self.style().right_margin()),
        )?;
        *height += 1;
        Ok(())
    }

    fn render_message(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        *height += self.style().slidebar_message_padding();
        if let Some(message) = self.message() {
            text::styled(
                app,
                message,
                &text::Style::new_with_colors(Set(self
                    .style()
                    .message_colors()))
                .with_align(1, 2)
                .with_top_margin(*height),
            )?;
            *height += 1;
        }
        Ok(())
    }

    fn render_back(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        *height += self.style().message_back_padding();
        text::styled(
            app,
            self.style().back_label(),
            &text::Style::new_with_colors(Set(self.style().back_colors()))
                .with_align(1, 2)
                .with_top_margin(*height),
        )?;
        *height += 1;
        Ok(())
    }
}
