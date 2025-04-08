pub use style::Style;
use thedes_tui_core::{
    App,
    event::{Event, Key},
    geometry::Coord,
    mutation::Set,
    screen::{self, FlushError},
};
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    cancellability::{Cancellation, NonCancellable},
    text,
};

mod style;

pub type KeyBindingMap = crate::key_bindings::KeyBindingMap<Command>;

pub fn default_key_bindings() -> KeyBindingMap {
    KeyBindingMap::new()
        .with(Key::Enter, Command::Confirm)
        .with(Key::Esc, Command::ConfirmCancel)
        .with(Key::Up, Command::ItemAbove)
        .with(Key::Down, Command::ItemBelow)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to render text")]
    RenderText(
        #[from]
        #[source]
        text::Error,
    ),
    #[error("Failed to flush tiles to canvas")]
    CanvasFlush(
        #[from]
        #[source]
        FlushError,
    ),
    #[error("Info dialog was cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Confirm,
    ConfirmCancel,
    ConfirmOk,
    ItemAbove,
    ItemBelow,
    SelectCancel,
    SelectOk,
}

#[derive(Debug, Clone)]
pub struct Info<C = NonCancellable> {
    title: String,
    message: String,
    cancellation: C,
    style: Style,
    key_bindings: KeyBindingMap,
}

impl Info {
    pub fn new(title: impl AsRef<str>, message: impl AsRef<str>) -> Self {
        Self::from_cancellation(title, message, NonCancellable)
    }
}

impl<C> Info<C>
where
    C: Cancellation<String>,
{
    pub fn from_cancellation(
        title: impl AsRef<str>,
        message: impl AsRef<str>,
        cancellation: C,
    ) -> Self {
        Self {
            title: title.as_ref().to_owned(),
            message: message.as_ref().to_owned(),
            cancellation,
            key_bindings: default_key_bindings(),
            style: Style::default(),
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

    pub fn with_message(mut self, message: &str) -> Self {
        self.set_message(message);
        self
    }

    pub fn set_message(&mut self, message: &str) -> &mut Self {
        self.message = message.to_owned();
        self
    }

    pub fn with_cancellation(mut self, cancellation: C) -> Self {
        self.set_cancellation(cancellation);
        self
    }

    pub fn set_cancellation(&mut self, cancellation: C) -> &mut Self {
        self.cancellation = cancellation;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.set_style(style);
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn key_bindings(&self) -> &KeyBindingMap {
        &self.key_bindings
    }

    pub fn cancellation(&self) -> &C {
        &self.cancellation
    }

    pub fn is_cancellable(&self) -> bool {
        self.cancellation().is_cancellable()
    }

    pub fn is_cancelling(&self) -> bool {
        self.cancellation().is_cancelling()
    }

    pub fn set_cancelling(&mut self, is_it: bool) {
        self.cancellation.set_cancelling(is_it);
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn run_command(&mut self, cmd: Command) -> Result<bool, Error> {
        match cmd {
            Command::Confirm => {
                return Ok(false);
            },
            Command::ConfirmCancel => {
                if self.is_cancellable() {
                    self.set_cancelling(true);
                    return Ok(false);
                }
            },
            Command::ConfirmOk => {
                self.set_cancelling(false);
                return Ok(false);
            },
            Command::ItemAbove | Command::SelectOk => {
                self.set_cancelling(false);
            },
            Command::ItemBelow | Command::SelectCancel => {
                self.set_cancelling(true);
            },
        }
        Ok(true)
    }

    pub async fn run(&mut self, app: &mut App) -> Result<(), Error> {
        while self.handle_input(app)? {
            self.render(app)?;
            tokio::select! {
                _ = app.tick_session.tick() => (),
                _ = app.cancel_token.cancelled() => Err(Error::Cancelled)?,
            }
        }
        Ok(())
    }

    fn handle_input(&mut self, app: &mut App) -> Result<bool, Error> {
        let Ok(mut events) = app.events.read_until_now() else {
            Err(Error::Cancelled)?
        };
        let mut should_continue = true;
        while let Some(event) = events.next().filter(|_| should_continue) {
            let Event::Key(key) = event else { continue };
            let Some(&command) = self.key_bindings.command_for(key) else {
                continue;
            };
            should_continue = self.run_command(command)?;
        }
        Ok(should_continue)
    }

    fn render(&mut self, app: &mut App) -> Result<(), Error> {
        app.canvas
            .queue([screen::Command::ClearScreen(self.style().background())]);

        let mut height = self.style().top_margin();
        self.render_title(app, &mut height)?;
        self.render_message(app, &mut height)?;
        self.render_ok(app, &mut height)?;
        self.render_cancel(app, &mut height)?;

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
        *height += self.style().title_message_padding();
        Ok(())
    }

    fn render_message(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        let mut bottom_margin = self.style().bottom_margin();
        bottom_margin += self.style().message_ok_padding();
        bottom_margin += 1;
        if self.is_cancellable() {
            bottom_margin += self.style().ok_cancel_padding();
            bottom_margin += 1;
        }
        bottom_margin -= 2;

        *height = text::styled(
            app,
            self.message(),
            &text::Style::new_with_colors(Set(self.style().title_colors()))
                .with_align(1, 2)
                .with_top_margin(*height)
                .with_left_margin(self.style().left_margin())
                .with_right_margin(self.style().right_margin())
                .with_bottom_margin(bottom_margin),
        )?;
        *height += self.style().message_ok_padding();
        Ok(())
    }

    fn render_ok(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        let graphemes = self.style().ok_label().graphemes(true).count();
        let right_padding = if graphemes % 2 == 0 { " " } else { "" };
        let rendered = format!("{}{}", self.style().ok_label(), right_padding);
        self.render_item(app, height, rendered, false)?;
        *height += 1;
        Ok(())
    }

    fn render_cancel(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        if self.is_cancellable() {
            *height += self.style().ok_cancel_padding();
            let graphemes = self.style().cancel_label().graphemes(true).count();
            let right_padding = if graphemes % 2 == 0 { " " } else { "" };
            let rendered =
                format!("{}{}", self.style().cancel_label(), right_padding);
            self.render_item(app, height, rendered, true)?;
        }
        Ok(())
    }

    fn render_item(
        &mut self,
        app: &mut App,
        height: &mut Coord,
        item: String,
        requires_cancelling: bool,
    ) -> Result<(), Error> {
        let is_selected = requires_cancelling == self.is_cancelling();
        let (colors, rendered) = if is_selected {
            let rendered = format!(
                "{}{}{}",
                self.style().selected_left(),
                item,
                self.style().selected_right(),
            );
            (self.style().selected_colors(), rendered)
        } else {
            (self.style().unselected_colors(), item)
        };
        text::styled(
            app,
            &rendered,
            &text::Style::new_with_colors(Set(colors))
                .with_align(1, 2)
                .with_top_margin(*height)
                .with_bottom_margin(app.canvas.size().y - *height - 2),
        )?;
        *height += 1;
        Ok(())
    }
}
