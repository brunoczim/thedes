//! This module exports a simple input dialog and related functionality.

use std::{iter, mem};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    color::{BasicColor, Color, ColorPair},
    event::{Event, Key, KeyEvent},
    geometry::{Coord, CoordPair},
    screen::TextStyle,
    RenderError,
    Tick,
};

/// A selected item/option of the input dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDialogItem {
    /// Input text prompt is going to be successful.
    Ok,
    /// Input text prompt is going to be cancelled.
    Cancel,
}

/// A dialog asking for user input, possibly filtered.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputDialog<F>
where
    F: FnMut(char) -> bool,
{
    filter: F,
    title: String,
    ok_label: String,
    cancel_label: String,
    buffer: String,
    max: Coord,
    title_colors: ColorPair,
    selected_colors: ColorPair,
    unselected_colors: ColorPair,
    cursor_colors: ColorPair,
    box_colors: ColorPair,
    bg: Color,
    title_y: Coord,
    pad_after_title: Coord,
    pad_after_box: Coord,
    pad_after_ok: Coord,
}

impl InputDialog<fn(char) -> bool> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            filter: |_| true,
            title: title.into(),
            ok_label: "OK".into(),
            cancel_label: "CANCEL".into(),
            buffer: String::new(),
            max: 32,
            title_colors: ColorPair::default(),
            selected_colors: !ColorPair::default(),
            unselected_colors: ColorPair::default(),
            cursor_colors: ColorPair::default(),
            box_colors: !ColorPair::default(),
            bg: BasicColor::Black.into(),
            title_y: 1,
            pad_after_title: 2,
            pad_after_box: 2,
            pad_after_ok: 1,
        }
    }
}

impl<F> InputDialog<F>
where
    F: Fn(char) -> bool,
{
    pub fn with_filter<F0>(self, new_filter: F0) -> InputDialog<F0>
    where
        F0: Fn(char) -> bool,
    {
        InputDialog {
            filter: new_filter,
            title: self.title,
            ok_label: self.ok_label,
            cancel_label: self.cancel_label,
            buffer: self.buffer,
            max: self.max,
            title_colors: self.title_colors,
            selected_colors: self.selected_colors,
            unselected_colors: self.unselected_colors,
            cursor_colors: self.cursor_colors,
            box_colors: self.box_colors,
            bg: self.bg,
            title_y: self.title_y,
            pad_after_title: self.pad_after_title,
            pad_after_box: self.pad_after_box,
            pad_after_ok: self.pad_after_ok,
        }
    }

    pub fn with_ok_label(self, label: impl Into<String>) -> Self {
        Self { ok_label: label.into(), ..self }
    }

    pub fn with_cancel_label(self, label: impl Into<String>) -> Self {
        Self { cancel_label: label.into(), ..self }
    }

    pub fn with_buffer_state(self, buffer: impl Into<String>) -> Self {
        Self { buffer: buffer.into(), ..self }
    }

    pub fn with_max_graphemes(self, label: impl Into<String>) -> Self {
        Self { cancel_label: label.into(), ..self }
    }

    pub fn with_title_colors(self, colors: ColorPair) -> Self {
        Self { title_colors: colors, ..self }
    }

    pub fn with_selected_colors(self, colors: ColorPair) -> Self {
        Self { selected_colors: colors, ..self }
    }

    pub fn with_unselected_colors(self, colors: ColorPair) -> Self {
        Self { unselected_colors: colors, ..self }
    }

    pub fn with_cursor_colors(self, colors: ColorPair) -> Self {
        Self { cursor_colors: colors, ..self }
    }

    pub fn with_box_colors(self, colors: ColorPair) -> Self {
        Self { box_colors: colors, ..self }
    }

    pub fn with_background(self, color: impl Into<Color>) -> Self {
        Self { bg: color.into(), ..self }
    }

    pub fn with_title_y(self, y: Coord) -> Self {
        Self { title_y: y, ..self }
    }

    pub fn with_pad_after_title(self, padding: Coord) -> Self {
        Self { pad_after_title: padding, ..self }
    }

    pub fn with_pad_after_box(self, padding: Coord) -> Self {
        Self { pad_after_box: padding, ..self }
    }

    pub fn with_pad_after_ok(self, padding: Coord) -> Self {
        Self { pad_after_ok: padding, ..self }
    }

    pub fn prompt(&self) -> Prompt<F> {
        self.prompt_with_initial(0)
    }

    pub fn prompt_with_initial(&self, cursor: usize) -> Prompt<F> {
        Prompt::without_cancel(self, cursor)
    }

    pub fn prompt_with_cancel(&self) -> Prompt<F> {
        self.prompt_cancel_initial(0, InputDialogItem::Ok)
    }

    pub fn prompt_cancel_initial(
        &self,
        cursor: usize,
        selected: InputDialogItem,
    ) -> Prompt<F> {
        Prompt::with_cancel(self, cursor, selected)
    }
}

pub struct Prompt<'dialog, F>
where
    F: FnMut(char) -> bool,
{
    dialog: &'dialog InputDialog<F>,
    buffer: Vec<char>,
    cursor: usize,
    selected: InputDialogItem,
    has_cancel: bool,
    actual_max: Coord,
    initialized: bool,
}

impl<'dialog, F> Prompt<'dialog, F>
where
    F: Fn(char) -> bool,
{
    /// Generic initialization. Should not be called directly, but through
    /// helpers.
    fn new(
        dialog: &'dialog InputDialog<F>,
        cursor: usize,
        selected: InputDialogItem,
        has_cancel: bool,
    ) -> Self {
        Self {
            buffer: dialog.buffer.chars().collect(),
            cursor,
            selected,
            has_cancel,
            actual_max: 0,
            dialog,
            initialized: false,
        }
    }

    /// Creates a selector from the given dialog and cursor position, without
    /// CANCEL.
    fn without_cancel(dialog: &'dialog InputDialog<F>, cursor: usize) -> Self {
        Self::new(dialog, cursor, InputDialogItem::Ok, false)
    }

    /// Creates a selector frm the given dialog, cursor position and selected
    /// item, with CANCEL being a possibility.
    fn with_cancel(
        dialog: &'dialog InputDialog<F>,
        cursor: usize,
        selected: InputDialogItem,
    ) -> Self {
        Self::new(dialog, cursor, selected, true)
    }

    pub fn on_tick(&mut self, tick: &mut Tick) -> Result<bool, RenderError> {
        if tick.screen().needs_resize() {
            return Ok(true);
        }

        if !self.initialized {
            self.init_run(&mut *tick)?;
        }

        while let Some(event) = tick.next_event() {
            match event {
                Event::Key(key_evt) => match key_evt {
                    KeyEvent {
                        main_key: Key::Up,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_up(&mut *tick)?,

                    KeyEvent {
                        main_key: Key::Down,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_down(&mut *tick)?,

                    KeyEvent {
                        main_key: Key::Esc,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } if self.has_cancel => {
                        if self.has_cancel {
                            self.selected = InputDialogItem::Cancel;
                            return Ok(false);
                        }
                    },

                    KeyEvent {
                        main_key: Key::Left,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_left(&mut *tick)?,

                    KeyEvent {
                        main_key: Key::Right,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_right(&mut *tick)?,

                    KeyEvent {
                        main_key: Key::Enter,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => return Ok(true),

                    KeyEvent {
                        main_key: Key::Backspace,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_backspace(&mut *tick)?,

                    KeyEvent {
                        main_key: Key::Char(ch),
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_char(&mut *tick, ch)?,

                    _ => (),
                },

                Event::Paste(paste_evt) => {
                    self.paste(&mut *tick, &paste_evt.data)?
                },
            }
        }

        Ok(true)
    }

    /// Computes the resulting string after accepting the dialog without the
    /// CANCEL option.
    pub fn result(&mut self) -> String {
        let buffer = mem::take(&mut self.buffer);
        buffer.into_iter().collect::<String>()
    }

    /// Computes the resulting string after accepting or rejecting the dialog
    /// (with the CANCEL option available).
    pub fn result_with_cancel(&mut self) -> Option<String> {
        match self.selected {
            InputDialogItem::Ok => Some(self.result()),
            InputDialogItem::Cancel => None,
        }
    }

    /// Initializes a run over this selector.
    fn init_run(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        self.selected = InputDialogItem::Ok;
        self.buffer = self.dialog.buffer.chars().collect::<Vec<_>>();
        self.cursor = 0;
        self.update_actual_max(tick.screen().canvas_size());
        self.render(&mut *tick)?;
        self.initialized = true;
        Ok(())
    }

    /// Updates the actual maximum length for the buffer, given a screen size.
    fn update_actual_max(&mut self, canvas_size: CoordPair) {
        self.actual_max = self.dialog.max.min(canvas_size.x);
        let max_index = usize::from(self.actual_max).saturating_sub(1);
        self.cursor = self.cursor.min(max_index);
        self.buffer.truncate(usize::from(self.actual_max));
    }

    /// Should be triggered when UP key is pressed.
    fn key_up(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        if self.has_cancel {
            self.selected = InputDialogItem::Ok;
            self.render_item(&mut *tick, InputDialogItem::Ok)?;
            self.render_item(&mut *tick, InputDialogItem::Cancel)?;
        }
        Ok(())
    }

    /// Should be triggered when DOWN key is pressed.
    fn key_down(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        if self.has_cancel {
            self.selected = InputDialogItem::Cancel;
            self.render_item(&mut *tick, InputDialogItem::Ok)?;
            self.render_item(&mut *tick, InputDialogItem::Cancel)?;
        }
        Ok(())
    }

    /// Should be triggered when LEFT key is pressed.
    fn key_left(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.render_input_box(&mut *tick)?;
        }
        Ok(())
    }

    /// Should be triggered when RIGHT key is pressed.
    fn key_right(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
            self.render_input_box(&mut *tick)?;
        }
        Ok(())
    }

    /// Should be triggered when BACKSPACE key is pressed.
    fn key_backspace(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
            self.render_input_box(&mut *tick)?;
        }
        Ok(())
    }

    fn paste(
        &mut self,
        tick: &mut Tick,
        contents: &str,
    ) -> Result<(), RenderError> {
        for ch in contents.chars() {
            self.insert(&mut *tick, ch)?;
        }
        Ok(())
    }

    /// Should be triggered when generic character key is pressed.
    fn key_char(
        &mut self,
        tick: &mut Tick,
        ch: char,
    ) -> Result<(), RenderError> {
        self.insert(tick, ch)?;
        Ok(())
    }

    fn insert(&mut self, tick: &mut Tick, ch: char) -> Result<(), RenderError> {
        if (self.dialog.filter)(ch) {
            let test_string = format!("a{}", ch);
            if test_string.graphemes(true).count() > 1 {
                let length = self.buffer.len() as Coord;
                if length < self.actual_max {
                    self.buffer.insert(self.cursor, ch);
                    self.cursor += 1;
                    self.render_input_box(&mut *tick)?;
                }
            }
        }
        Ok(())
    }

    /// Renders the whole input dialog.
    fn render(&self, tick: &mut Tick) -> Result<(), RenderError> {
        tick.screen_mut().clear_canvas(self.dialog.bg)?;
        self.render_title(&mut *tick)?;
        self.render_input_box(&mut *tick)?;
        self.render_item(&mut *tick, InputDialogItem::Ok)?;
        if self.has_cancel {
            self.render_item(&mut *tick, InputDialogItem::Cancel)?;
        }
        Ok(())
    }

    /// Renders the title of the input dialog.
    fn render_title(&self, tick: &mut Tick) -> Result<(), RenderError> {
        let style = TextStyle::default()
            .with_left_margin(1)
            .with_right_margin(1)
            .with_align(1, 2)
            .with_max_height(self.dialog.pad_after_title.saturating_add(1))
            .with_top_margin(self.dialog.title_y);
        tick.screen_mut().print(&self.dialog.title, &style)?;
        Ok(())
    }

    /// Renders the input box of the input dialog.
    fn render_input_box(&self, tick: &mut Tick) -> Result<(), RenderError> {
        let mut field = self.buffer.iter().collect::<String>();
        let additional = usize::from(self.actual_max) - self.buffer.len();
        field.extend(iter::repeat(' ').take(additional));

        let style = TextStyle::default()
            .with_align(1, 2)
            .with_top_margin(self.y_of_box())
            .with_colors(self.dialog.box_colors);
        tick.screen_mut().print(&field, &style)?;

        let width = tick.screen().canvas_size().x;
        let correction = usize::from(self.actual_max % 2 + width % 2 + 1);
        let length = field.len() - correction % 2;

        field.clear();
        for i in 0 .. length + 1 {
            if i == self.cursor {
                field.push('¯')
            } else {
                field.push(' ')
            }
        }

        let style = TextStyle::default()
            .with_align(1, 2)
            .with_top_margin(self.y_of_box() + 1)
            .with_left_margin(1)
            .with_colors(self.dialog.cursor_colors);
        tick.screen_mut().print(&field, &style)?;

        Ok(())
    }

    /// Renders an item/option of the input dialog.
    fn render_item(
        &self,
        tick: &mut Tick,
        item: InputDialogItem,
    ) -> Result<(), RenderError> {
        let (option, y) = match item {
            InputDialogItem::Ok => (&self.dialog.ok_label, self.y_of_ok()),
            InputDialogItem::Cancel => {
                (&self.dialog.cancel_label, self.y_of_cancel())
            },
        };
        let colors = if item == self.selected {
            self.dialog.selected_colors
        } else {
            self.dialog.unselected_colors
        };

        let label = format!("> {} <", option);

        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(colors)
            .with_top_margin(y);
        tick.screen_mut().print(&label, &style)?;

        Ok(())
    }

    /// Computes the Y coordinate of the input box.
    fn y_of_box(&self) -> Coord {
        self.dialog.title_y + 1 + self.dialog.pad_after_title
    }

    /// Computes the Y coordinate of the OK option.
    fn y_of_ok(&self) -> Coord {
        self.y_of_box() + 2 + self.dialog.pad_after_box
    }

    /// Computes the Y coordinate of the CANCEL option.
    fn y_of_cancel(&self) -> Coord {
        self.y_of_ok() + 1 + self.dialog.pad_after_ok
    }
}
