use std::iter;

pub use style::Style;
use thedes_tui_core::{
    App,
    event::{Event, Key, KeyEvent},
    geometry::Coord,
    mutation::Set,
    screen::{self, FlushError},
};
use thiserror::Error;

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
        .with(Key::Left, Command::MoveLeft)
        .with(Key::Right, Command::MoveRight)
        .with(Key::Backspace, Command::DeleteBehind)
        .with(Key::Delete, Command::DeleteAhead)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid zero maximum input length")]
    InvalidZeroMax,
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
    #[error("Menu was cancelled")]
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
    MoveLeft,
    MoveRight,
    Move(Coord),
    Insert(char),
    DeleteAhead,
    DeleteBehind,
}

#[derive(Debug, Clone)]
pub struct Config<'a, F> {
    pub filter: F,
    pub max: Coord,
    pub title: &'a str,
}

#[derive(Debug, Clone)]
pub struct Input<F = fn(char) -> bool, C = NonCancellable> {
    filter: F,
    title: String,
    max: Coord,
    buffer: Vec<char>,
    cancellation: C,
    cursor: Coord,
    style: Style,
    key_bindings: KeyBindingMap,
}

impl<F> Input<F>
where
    F: FnMut(char) -> bool,
{
    pub fn new(config: Config<'_, F>) -> Result<Self, Error> {
        Self::from_cancellation(config, NonCancellable)
    }
}

impl<F, C> Input<F, C>
where
    F: FnMut(char) -> bool,
    C: Cancellation<String>,
{
    pub fn from_cancellation(
        config: Config<'_, F>,
        cancellation: C,
    ) -> Result<Self, Error> {
        if config.max == 0 {
            Err(Error::InvalidZeroMax)?
        }
        Ok(Self {
            filter: config.filter,
            title: config.title.to_owned(),
            max: config.max,
            cancellation,
            key_bindings: default_key_bindings(),
            style: Style::default(),
            buffer: Vec::with_capacity(usize::from(config.max)),
            cursor: 0,
        })
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.set_title(title);
        self
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_owned();
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

    pub fn with_max(mut self, max: Coord) -> Result<Self, Error> {
        self.set_max(max)?;
        Ok(self)
    }

    pub fn set_max(&mut self, max: Coord) -> Result<&mut Self, Error> {
        if max == 0 {
            Err(Error::InvalidZeroMax)?
        }
        self.max = max;
        self.buffer.truncate(usize::from(max));
        Ok(self)
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.set_style(style);
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    pub fn with_buffer<I>(mut self, chars: I) -> Result<Self, usize>
    where
        I: IntoIterator<Item = char>,
    {
        self.set_buffer(chars)?;
        Ok(self)
    }

    pub fn set_buffer<I>(&mut self, chars: I) -> Result<&mut Self, usize>
    where
        I: IntoIterator<Item = char>,
    {
        self.clear_buffer();
        self.insert_chars(chars)?;
        Ok(self)
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

    pub fn finish_buffer(&self) -> String {
        self.buffer.iter().copied().collect()
    }

    pub fn output(&self) -> C::Output {
        self.cancellation().make_output(self.finish_buffer())
    }

    pub fn insert_char(&mut self, char: char) {
        if (self.filter)(char) {
            if self.len() < self.max() {
                self.buffer.insert(usize::from(self.cursor), char);
                self.cursor += 1;
            }
        }
    }

    pub fn delete_behind(&mut self) {
        if self.len() > 0 {
            self.cursor -= 1;
            self.buffer.remove(usize::from(self.cursor()));
        }
    }

    pub fn delete_ahead(&mut self) {
        if self.len() > 0 && self.cursor() < self.len() {
            self.buffer.remove(usize::from(self.cursor()));
            self.cursor = self.cursor().min(self.len().saturating_sub(1));
        }
    }

    pub fn insert_chars<I>(&mut self, chars: I) -> Result<(), usize>
    where
        I: IntoIterator<Item = char>,
    {
        let mut chars = chars.into_iter();
        while self.buffer.len() < usize::from(self.max()) {
            let Some(char) = chars.next() else { break };
            self.insert_char(char);
        }
        let chars_left = chars.count();
        if chars_left == 0 { Ok(()) } else { Err(chars_left) }
    }

    pub fn move_left(&mut self) {
        let _ = self.set_cursor(self.cursor().saturating_sub(1));
    }

    pub fn move_right(&mut self) {
        let _ = self.set_cursor(self.cursor().saturating_add(1));
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn max(&self) -> Coord {
        self.max
    }

    pub fn len(&self) -> Coord {
        self.buffer.len() as Coord
    }

    pub fn cursor(&self) -> Coord {
        self.cursor
    }

    pub fn set_cursor(&mut self, position: Coord) -> Result<(), Coord> {
        self.cursor = position.min(self.len());
        if position == self.cursor {
            Ok(())
        } else {
            Err(position - self.cursor)
        }
    }

    pub fn clear_buffer(&mut self) {
        let _ = self.set_cursor(0);
        self.buffer.clear();
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
            Command::Insert(ch) => self.insert_char(ch),
            Command::DeleteBehind => self.delete_behind(),
            Command::DeleteAhead => self.delete_ahead(),
            Command::MoveLeft => self.move_left(),
            Command::MoveRight => self.move_right(),
            Command::Move(i) => {
                let _ = self.set_cursor(i);
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
            if let KeyEvent {
                main_key: Key::Char(ch),
                alt: false,
                ctrl: false,
                shift: false,
            } = key
            {
                self.insert_char(ch);
            } else {
                let Some(&command) = self.key_bindings.command_for(key) else {
                    continue;
                };
                should_continue = self.run_command(command)?;
            }
        }
        Ok(should_continue)
    }

    fn render(&mut self, app: &mut App) -> Result<(), Error> {
        app.canvas
            .queue([screen::Command::ClearScreen(self.style().background())]);

        let mut height = self.style().top_margin();
        self.render_title(app, &mut height)?;

        self.render_field(app, &mut height)?;
        self.render_cursor(app, &mut height)?;
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
        *height += self.style().title_field_padding();
        Ok(())
    }

    fn render_field(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        let padding_len = usize::from(self.max() - self.len());
        let field_chars: String = self
            .buffer
            .iter()
            .copied()
            .chain(iter::repeat(' ').take(padding_len))
            .collect();
        text::styled(
            app,
            &field_chars,
            &text::Style::new_with_colors(Set(self.style().field_colors()))
                .with_align(1, 2)
                .with_top_margin(*height)
                .with_left_margin(self.style().left_margin())
                .with_right_margin(self.style().right_margin()),
        )?;
        *height += 1;
        Ok(())
    }

    fn render_cursor(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        let prefix_len = usize::from(self.cursor() + 1);
        let suffix_len = usize::from(self.max() - self.cursor());
        let cursor_chars: String = iter::repeat(' ')
            .take(prefix_len)
            .chain(iter::once(self.style().cursor()))
            .chain(iter::repeat(' ').take(suffix_len))
            .collect();
        text::styled(
            app,
            &cursor_chars,
            &text::Style::new_with_colors(Set(self.style().cursor_colors()))
                .with_align(1, 2)
                .with_top_margin(*height)
                .with_left_margin(self.style().left_margin())
                .with_right_margin(self.style().right_margin()),
        )?;
        *height += 1;
        *height += self.style().field_ok_padding();
        Ok(())
    }

    fn render_ok(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        self.render_item(
            app,
            height,
            self.style().ok_label().to_owned(),
            false,
        )?;
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
            self.render_item(
                app,
                height,
                self.style().cancel_label().to_owned(),
                true,
            )?;
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
