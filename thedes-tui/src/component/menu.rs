use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::{
    color::{BasicColor, Color, ColorPair},
    event::{Event, Key, KeyEvent},
    geometry::{Coord, CoordPair},
    style::TextStyle,
    RenderError,
    Tick,
};

#[derive(Debug, Clone)]
pub struct Menu<O> {
    title: String,
    options: Vec<MenuOption<O>>,
    cancel_label: String,
    title_colors: ColorPair,
    arrow_colors: ColorPair,
    selected_colors: ColorPair,
    unselected_colors: ColorPair,
    background: Color,
    title_y: Coord,
    pad_after_title: Coord,
    pad_after_option: Coord,
}

impl<O> Menu<O> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            options: Vec::new(),
            cancel_label: "CANCEL".into(),
            title_colors: ColorPair::default(),
            arrow_colors: ColorPair::default(),
            selected_colors: !ColorPair::default(),
            unselected_colors: ColorPair::default(),
            background: BasicColor::Black.into(),
            title_y: 1,
            pad_after_title: 2,
            pad_after_option: 1,
        }
    }

    pub fn add_option(mut self, item: O, rendered: impl Into<String>) -> Self {
        self.options.push(MenuOption { item, rendered: rendered.into() });
        self
    }

    pub fn with_cancel_label(self, label: impl Into<String>) -> Self {
        Self { cancel_label: label.into(), ..self }
    }

    pub fn with_title_colors(self, colors: ColorPair) -> Self {
        Self { title_colors: colors, ..self }
    }

    pub fn with_arrow_colors(self, colors: ColorPair) -> Self {
        Self { arrow_colors: colors, ..self }
    }

    pub fn with_selected_colors(self, colors: ColorPair) -> Self {
        Self { selected_colors: colors, ..self }
    }

    pub fn with_unselected_colors(self, colors: ColorPair) -> Self {
        Self { unselected_colors: colors, ..self }
    }

    pub fn with_background(self, color: impl Into<Color>) -> Self {
        Self { background: color.into(), ..self }
    }

    pub fn with_title_y(self, offset: Coord) -> Self {
        Self { title_y: offset, ..self }
    }

    pub fn with_pad_after_title(self, padding: Coord) -> Self {
        Self { pad_after_title: padding, ..self }
    }

    pub fn with_pad_after_option(self, padding: Coord) -> Self {
        Self { pad_after_option: padding, ..self }
    }

    pub fn select(&self) -> Selector<O> {
        self.select_with_initial(0)
    }

    pub fn select_with_initial(&self, initial: usize) -> Selector<O> {
        Selector::without_cancel(self, initial)
    }

    pub fn select_with_cancel(&self) -> Selector<O> {
        self.select_cancel_initial(0, false)
    }

    pub fn select_cancel_initial(
        &self,
        initial: usize,
        cancel: bool,
    ) -> Selector<O> {
        Selector::with_cancel(self, initial, cancel)
    }
}

#[derive(Debug, Clone)]
struct MenuOption<O> {
    item: O,
    rendered: String,
}

#[derive(Debug, Clone)]
pub enum Selection<O> {
    Option(O),
    Cancel,
}

/// Menu selection runner.
#[derive(Debug, Clone)]
pub struct Selector<'menu, O> {
    /// A reference to the original menu.
    menu: &'menu Menu<O>,
    /// First row currently shown.
    first_row: usize,
    /// Last row currently shown.
    last_row: usize,
    /// Row currently selected (or that was previously selected before the
    /// cancel being currently selected).
    selected: usize,
    /// Whether the cancel option is currently selected, IF cancel is `Some`.
    cancel: Option<bool>,
    /// Whether it was initialized or not.
    initialized: bool,
}

impl<'menu, O> Selector<'menu, O> {
    /// Generic initialization for this selector, do not call directly unless
    /// really needed and wrapped.
    fn new(menu: &'menu Menu<O>, initial: usize, cancel: Option<bool>) -> Self {
        Selector {
            menu,
            selected: initial,
            cancel,
            first_row: 0,
            last_row: 0,
            initialized: false,
        }
    }

    /// Initializes this selector for a selection without cancel option.
    fn without_cancel(menu: &'menu Menu<O>, initial: usize) -> Self {
        Self::new(menu, initial, None)
    }

    /// Initializes this selector for a selection with cancel option.
    fn with_cancel(menu: &'menu Menu<O>, initial: usize, cancel: bool) -> Self {
        Selector::new(menu, initial, Some(cancel || menu.options.len() == 0))
    }

    pub fn selection(&self) -> &O {
        &self.menu.options[self.selected].item
    }

    pub fn cancellable_selection(&self) -> Selection<&O> {
        if self.cancel == Some(true) {
            Selection::Cancel
        } else {
            Selection::Option(self.selection())
        }
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
                Event::Key(keys) => match keys {
                    KeyEvent {
                        main_key: Key::Esc,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => {
                        if let Some(cancel) = self.cancel.as_mut() {
                            *cancel = true;
                            return Ok(false);
                        }
                    },

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
                    } => return Ok(false),

                    _ => (),
                },

                _ => (),
            }
        }

        Ok(true)
    }

    /// Initializes the run of this selector.
    fn init_run(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        self.render(&mut *tick)?;
        self.update_last_row(tick.screen().canvas_size());
        self.initialized = true;
        Ok(())
    }

    /// Should be triggered when UP key is pressed.
    fn key_up(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        if self.is_cancelling() && self.menu.options.len() > 0 {
            self.cancel = Some(false);
            self.render(&mut *tick)?;
        } else if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.first_row {
                self.first_row -= 1;
                self.update_last_row(tick.screen().canvas_size());
            }
            self.render(&mut *tick)?;
        }
        Ok(())
    }

    /// Should be triggered when DOWN key is pressed.
    fn key_down(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        if self.selected + 1 < self.menu.options.len() {
            self.selected += 1;
            if self.selected >= self.last_row {
                self.first_row += 1;
                self.update_last_row(tick.screen().canvas_size());
            }
            self.render(&mut *tick)?;
        } else if self.is_not_cancelling() {
            self.cancel = Some(true);
            self.render(&mut *tick)?;
        }
        Ok(())
    }

    /// Should be triggered when LEFT key is pressed.
    fn key_left(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        if self.is_not_cancelling() {
            self.cancel = Some(true);
            self.render(&mut *tick)?;
        }
        Ok(())
    }

    /// Should be triggered when RIGHT key is pressed.
    fn key_right(&mut self, tick: &mut Tick) -> Result<(), RenderError> {
        if self.is_cancelling() && self.menu.options.len() > 0 {
            self.cancel = Some(false);
            self.render(&mut *tick)?;
        }
        Ok(())
    }

    /// Returns if the selection is currently selecting the cancel option.
    fn is_cancelling(&self) -> bool {
        self.cancel == Some(true)
    }

    /// Returns if the selection is currently not selecting the cancel option
    /// AND the cancel option is enabled.
    fn is_not_cancelling(&self) -> bool {
        self.cancel == Some(false)
    }

    /// Updates the last row field from the computed end of the screen.
    fn update_last_row(&mut self, canvas_size: CoordPair) {
        self.last_row = self.screen_end(canvas_size);
    }

    /// Returns the index of the last visible option in the screen.
    fn screen_end(&self, canvas_size: CoordPair) -> usize {
        let cancel = if self.cancel.is_some() { 4 } else { 0 };
        let mut available = canvas_size.y - self.menu.title_y;
        available -= 2 * (self.menu.pad_after_title - 1) + cancel;
        let extra = available / (self.menu.pad_after_option + 1) - 2;
        self.first_row + usize::from(extra)
    }

    /// Returns the range of the visible options in the screen.
    fn range_of_screen(&self, canvas_size: CoordPair) -> Range<usize> {
        self.first_row .. self.screen_end(canvas_size)
    }

    /// Renders the whole menu.
    fn render(&self, tick: &mut Tick) -> Result<(), RenderError> {
        tick.screen_mut().clear_canvas(self.menu.background)?;
        self.render_title(&mut *tick)?;

        let arrow_style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.menu.arrow_colors);

        let mut range = self.range_of_screen(tick.screen().canvas_size());
        self.render_up_arrow(&mut *tick, &arrow_style)?;
        self.render_down_arrow(&mut *tick, &arrow_style, &mut range)?;

        self.render_options(&mut *tick, range)?;
        let canvas_size = tick.screen().canvas_size();
        self.render_cancel(&mut *tick, canvas_size.y)?;

        Ok(())
    }

    /// Renders the title of the menu.
    fn render_title(&self, tick: &mut Tick) -> Result<(), RenderError> {
        let title_style = TextStyle::default()
            .with_align(1, 2)
            .with_top_margin(self.menu.title_y)
            .with_colors(self.menu.title_colors)
            .with_max_height(self.menu.pad_after_title.saturating_add(1));
        tick.screen_mut().print(&self.menu.title, &title_style)?;
        Ok(())
    }

    /// Renders the UP arrow.
    fn render_up_arrow(
        &self,
        tick: &mut Tick,
        style: &TextStyle,
    ) -> Result<(), RenderError> {
        if self.first_row > 0 {
            let mut option_y = self.y_of_option(self.first_row);
            option_y -= self.menu.pad_after_option + 1;
            let style = style.with_top_margin(option_y);
            tick.screen_mut().print("Ʌ", &style)?;
        }
        Ok(())
    }

    /// Renders the DOWN arrow and updates the given range of the screen.
    fn render_down_arrow(
        &self,
        tick: &mut Tick,
        style: &TextStyle,
        range: &mut Range<usize>,
    ) -> Result<(), RenderError> {
        if range.end < self.menu.options.len() {
            let option_y = self.y_of_option(range.end);
            let style = style.with_top_margin(option_y);
            tick.screen_mut().print("V", &style)?;
        } else {
            range.end = self.menu.options.len();
        }

        Ok(())
    }

    /// Renders all the options of the given range.
    fn render_options(
        &self,
        tick: &mut Tick,
        range: Range<usize>,
    ) -> Result<(), RenderError> {
        for (i, option) in self.menu.options[range.clone()].iter().enumerate() {
            let is_selected =
                range.start + i == self.selected && !self.is_cancelling();

            self.render_option(
                &mut *tick,
                option,
                self.y_of_option(range.start + i),
                is_selected,
            )?;
        }

        Ok(())
    }

    /// Renders a single option.
    fn render_option(
        &self,
        tick: &mut Tick,
        option: &MenuOption<O>,
        option_y: Coord,
        selected: bool,
    ) -> Result<(), RenderError> {
        let mut buf = option.rendered.clone();
        let mut len = buf.graphemes(true).count();
        let canvas_size = tick.screen().canvas_size();

        if (len % 2) as Coord != canvas_size.x % 2 {
            buf += " ";
            len += 1;
        }

        if usize::from(canvas_size.x - 4) < len {
            buf = buf.graphemes(true).take(len - 5).collect();
            buf += "…";
        }

        buf = format!("> {buf} <");

        let colors = if selected {
            self.menu.selected_colors
        } else {
            self.menu.unselected_colors
        };
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(colors)
            .with_top_margin(option_y);
        tick.screen_mut().print(&buf, &style)?;

        Ok(())
    }

    /// Renders the cancel option, if any.
    fn render_cancel(
        &self,
        tick: &mut Tick,
        cancel_y: Coord,
    ) -> Result<(), RenderError> {
        if let Some(selected) = self.cancel {
            let colors = if selected {
                self.menu.selected_colors
            } else {
                self.menu.unselected_colors
            };

            let style = TextStyle::default()
                .with_align(1, 3)
                .with_colors(colors)
                .with_top_margin(cancel_y - 2);
            let label_string = format!("> {} <", &self.menu.cancel_label);
            tick.screen_mut().print(&label_string, &style)?;
        }

        Ok(())
    }

    /// Gets the height of a given option (by index).
    fn y_of_option(&self, option: usize) -> Coord {
        let count = (option - self.first_row) as Coord;
        let before = (count + 1) * (self.menu.pad_after_option + 1);
        before + self.menu.pad_after_title + 1 + self.menu.title_y
    }
}

/// An item of a prompt about a dangerous action.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DangerPromptOption {
    /// Returned when user cancels this action.
    Cancel,
    /// Returned when user confirms this action.
    Ok,
}

impl DangerPromptOption {
    pub fn menu(title: impl Into<String>) -> Menu<Self> {
        Menu::new(title)
            .add_option(Self::Ok, "OK")
            .add_option(Self::Cancel, "CANCEL")
    }
}
