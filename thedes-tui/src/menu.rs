use std::fmt;

use thedes_tui_core::{
    App,
    event::{Event, Key},
    geometry::Coord,
    mutation::Set,
    screen::{self, FlushError},
};
use thiserror::Error;

use crate::{
    cancellability::{Cancellation, NonCancellable},
    text,
};

pub use style::Style;

mod style;

pub fn default_key_bindings() -> KeyBindingMap {
    KeyBindingMap::new()
        .with(Key::Enter, Command::Confirm)
        .with(Key::Esc, Command::ConfirmCancel)
        .with(Key::Char('q'), Command::ConfirmCancel)
        .with(Key::Up, Command::ItemAbove)
        .with(Key::Down, Command::ItemBelow)
        .with(Key::Left, Command::UnsetCancelling)
        .with(Key::Right, Command::SetCancelling)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Cannot create menu with no items")]
    NoItems,
    #[error("Requested index {given} out of bounds with length {len}")]
    OutOfBounds { len: usize, given: usize },
    #[error("Requested scroll top index will exclude selected index")]
    ScrollExcludesSelected,
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

pub type KeyBindingMap = crate::key_bindings::KeyBindingMap<Command>;

pub trait Item: fmt::Display {}

impl<T> Item for T where T: fmt::Display {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Confirm,
    ConfirmCancel,
    ConfirmItem,
    ItemAbove,
    ItemBelow,
    SetCancelling,
    UnsetCancelling,
    Select(usize),
    SelectConfirm(usize),
}

#[derive(Debug, Clone)]
pub struct Menu<'a, I, C = NonCancellable> {
    title: &'a str,
    items: &'a [I],
    selected: usize,
    scroll: usize,
    style: Style,
    cancellation: C,
    key_bindings: KeyBindingMap,
}

impl<'a, I> Menu<'a, I>
where
    I: Item,
{
    pub fn new(title: &'a str, items: &'a [I]) -> Result<Self, Error> {
        Self::from_cancellation(title, items, NonCancellable)
    }
}

impl<'a, I, C> Menu<'a, I, C>
where
    I: Item,
    C: Cancellation<&'a I>,
{
    pub fn from_cancellation(
        title: &'a str,
        items: &'a [I],
        cancellation: C,
    ) -> Result<Self, Error> {
        if items.is_empty() {
            Err(Error::NoItems)?
        }
        Ok(Self {
            title,
            items,
            scroll: 0,
            selected: 0,
            style: Style::default(),
            cancellation,
            key_bindings: default_key_bindings(),
        })
    }

    pub fn with_title(mut self, title: &'a str) -> Self {
        self.set_title(title);
        self
    }

    pub fn set_title(&mut self, title: &'a str) -> &mut Self {
        self.title = title;
        self
    }

    pub fn with_items(mut self, items: &'a [I]) -> Result<Self, Error> {
        self.set_items(items)?;
        Ok(self)
    }

    pub fn set_items(&mut self, items: &'a [I]) -> Result<&mut Self, Error> {
        if items.is_empty() {
            Err(Error::NoItems)?
        }
        self.items = items;
        self.selected = self.selected.min(items.len() - 1);
        Ok(self)
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

    pub fn with_keybindings(mut self, map: KeyBindingMap) -> Self {
        self.set_keybindings(map);
        self
    }

    pub fn set_keybindings(&mut self, map: KeyBindingMap) -> &mut Self {
        self.key_bindings = map;
        self
    }

    pub fn with_selected(mut self, index: usize) -> Result<Self, Error> {
        self.set_selected(index)?;
        Ok(self)
    }

    pub fn set_selected(&mut self, index: usize) -> Result<&mut Self, Error> {
        if index >= self.items.len() {
            Err(Error::OutOfBounds { len: self.items.len(), given: index })?
        }
        self.selected = index;
        Ok(self)
    }

    pub fn with_scroll(mut self, top_item_index: usize) -> Result<Self, Error> {
        self.set_scroll(top_item_index)?;
        Ok(self)
    }

    pub fn set_scroll(
        &mut self,
        top_item_index: usize,
    ) -> Result<&mut Self, Error> {
        if top_item_index >= self.items.len() {
            Err(Error::OutOfBounds {
                len: self.items.len(),
                given: top_item_index,
            })?
        }
        if top_item_index > self.selected_index() {
            Err(Error::ScrollExcludesSelected)?;
        }
        self.scroll = top_item_index;
        Ok(self)
    }

    pub fn title(&self) -> &'a str {
        self.title
    }

    pub fn items(&self) -> &'a [I] {
        self.items
    }

    pub fn cancellation(&self) -> &C {
        &self.cancellation
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn key_bindings(&self) -> &KeyBindingMap {
        &self.key_bindings
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn scroll_top_index(&self) -> usize {
        self.scroll
    }

    pub fn selected_item(&self) -> &'a I {
        &self.items[self.selected]
    }

    pub fn output(&self) -> C::Output {
        self.cancellation().make_output(self.selected_item())
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

    pub fn run_command(&mut self, cmd: Command) -> Result<bool, Error> {
        match cmd {
            Command::Confirm => return Ok(false),
            Command::ConfirmItem => {
                self.set_cancelling(false);
                return Ok(false);
            },
            Command::ConfirmCancel => {
                if self.is_cancellable() {
                    self.set_cancelling(true);
                    return Ok(false);
                }
            },
            Command::ItemAbove => {
                let new_index = self.selected_index().saturating_sub(1);
                self.set_selected(new_index)?;
            },
            Command::ItemBelow => {
                let new_index = self
                    .selected_index()
                    .saturating_add(1)
                    .min(self.items().len() - 1);
                self.set_selected(new_index)?;
            },
            Command::SetCancelling => {
                self.set_cancelling(true);
            },
            Command::UnsetCancelling => {
                self.set_cancelling(false);
            },
            Command::Select(index) => {
                self.set_selected(index)?;
            },
            Command::SelectConfirm(index) => {
                self.set_selected(index)?;
                return Ok(false);
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

        let scroll_bottom_index = self.compute_scroll_bottom(app, height)?;

        self.render_top_arrow(app, &mut height)?;
        self.render_items(app, &mut height, scroll_bottom_index)?;
        self.render_bottom_arrow(app, &mut height, scroll_bottom_index)?;
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
        *height += self.style().title_top_arrow_padding();
        Ok(())
    }

    fn compute_scroll_bottom(
        &mut self,
        app: &mut App,
        height: Coord,
    ) -> Result<usize, Error> {
        let max_items_rem = self.items().len() - self.scroll_top_index();
        let mut available_screen = app.canvas.size().y;
        available_screen -= height;
        available_screen -= self.style().bottom_margin();
        available_screen -= self.style().items_bottom_arrow_padding();
        available_screen -= 1;
        if self.is_cancellable() {
            available_screen -= self.style().bottom_arrow_cancel_padding();
            available_screen -= 1;
        }
        let item_required_space = 1 + self.style().item_between_padding();
        let max_displayable_items = usize::from(
            (available_screen + item_required_space - 1) / item_required_space,
        );
        let scroll_count = max_items_rem.min(max_displayable_items);
        let mut scroll_bottom_index = self.scroll_top_index() + scroll_count;
        if self.selected_index() >= scroll_bottom_index {
            scroll_bottom_index = self.selected_index() + 1;
            self.set_scroll(scroll_bottom_index - scroll_count)?;
        }
        Ok(scroll_bottom_index)
    }

    fn render_top_arrow(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        if self.scroll_top_index() > 0 {
            let colors = self.style().top_arrow_colors();
            text::styled(
                app,
                self.style().top_arrow(),
                &text::Style::new_with_colors(Set(colors))
                    .with_align(1, 2)
                    .with_top_margin(*height)
                    .with_bottom_margin(app.canvas.size().y - *height - 2),
            )?;
        }

        *height += 1;
        *height += self.style().top_arrow_items_padding();

        Ok(())
    }

    fn render_items(
        &mut self,
        app: &mut App,
        height: &mut Coord,
        scroll_bottom_index: usize,
    ) -> Result<(), Error> {
        for (scroll_i, i) in
            (self.scroll_top_index() .. scroll_bottom_index).enumerate()
        {
            if scroll_i > 0 {
                *height += self.style().item_between_padding();
            }
            self.render_item(app, height, i)?;
        }
        Ok(())
    }

    fn render_item(
        &mut self,
        app: &mut App,
        height: &mut Coord,
        index: usize,
    ) -> Result<(), Error> {
        let is_selected =
            index == self.selected_index() && !self.is_cancelling();
        let (colors, rendered) = if is_selected {
            let rendered = format!(
                "{}{}{}",
                self.style().selected_left(),
                self.items()[index],
                self.style().selected_right(),
            );
            (self.style().selected_colors(), rendered)
        } else {
            let rendered = format!("{}", self.items()[index]);
            (self.style().unselected_colors(), rendered)
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

    fn render_bottom_arrow(
        &mut self,
        app: &mut App,
        height: &mut Coord,
        scroll_bottom_index: usize,
    ) -> Result<(), Error> {
        *height += self.style().items_bottom_arrow_padding();
        if scroll_bottom_index > self.items().len() {
            let colors = self.style().bottom_arrow_colors();
            text::styled(
                app,
                self.style().bottom_arrow(),
                &text::Style::new_with_colors(Set(colors))
                    .with_align(1, 2)
                    .with_top_margin(*height)
                    .with_bottom_margin(app.canvas.size().y - *height - 2),
            )?;
        }

        Ok(())
    }

    fn render_cancel(
        &mut self,
        app: &mut App,
        height: &mut Coord,
    ) -> Result<(), Error> {
        if self.is_cancellable() {
            *height += self.style().bottom_arrow_cancel_padding();
            let is_selected = self.is_cancelling();
            let (colors, rendered) = if is_selected {
                let rendered = format!(
                    "{}{}{}",
                    self.style().selected_left(),
                    self.style().cancel_label(),
                    self.style().selected_right(),
                );
                (self.style().selected_colors(), rendered)
            } else {
                let rendered = format!("{}", self.selected_item());
                (self.style().unselected_colors(), rendered)
            };
            text::styled(
                app,
                &rendered,
                &text::Style::new_with_colors(Set(colors))
                    .with_align(1, 1)
                    .with_top_margin(*height)
                    .with_bottom_margin(app.canvas.size().y - *height - 2),
            )?;
        }

        Ok(())
    }
}
