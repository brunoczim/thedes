use std::{collections::BTreeSet, fmt, hash::Hash, ops::Range};

use indexmap::IndexMap;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    color::{BasicColor, Color, ColorPair},
    event::{Event, Key, KeyEvent},
    geometry::{Coord, CoordPair},
    screen::TextStyle,
    CanvasError,
    Tick,
};

use super::SelectionCancellability;

#[derive(Debug, Error)]
#[error("Option {0} is not in the menu")]
pub struct UnknownOption<O>(pub O);

pub trait OptionItem: Hash + Ord + Clone + fmt::Debug + fmt::Display {}

#[derive(Debug, Clone)]
pub struct Options<O> {
    set: BTreeSet<O>,
    initial: O,
}

impl<O> Options<O>
where
    O: OptionItem,
{
    pub fn with_initial(initial: O) -> Self {
        Self { set: BTreeSet::from([initial.clone()]), initial }
    }

    pub fn add(mut self, option: O) -> Self {
        self.set.insert(option);
        self
    }
}

#[derive(Debug, Clone)]
pub struct BaseConfig {
    title: String,
    title_colors: ColorPair,
    arrow_colors: ColorPair,
    selected_colors: ColorPair,
    unselected_colors: ColorPair,
    background: Color,
    title_y: Coord,
    pad_after_title: Coord,
    pad_after_option: Coord,
}

impl BaseConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
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
}

#[derive(Debug, Clone)]
pub struct Config<O, C> {
    pub base: BaseConfig,
    pub options: Options<O>,
    pub cancellability: C,
}

/// Menu selection runner.
#[derive(Debug, Clone)]
pub struct Menu<O, C> {
    options: IndexMap<O, String>,
    selected: usize,
    cancellability: C,
    base_config: BaseConfig,
    first_row: usize,
    last_row: usize,
}

impl<O, C> Menu<O, C>
where
    O: OptionItem,
    for<'a> C: SelectionCancellability<&'a O>,
{
    pub fn new(config: Config<O, C>) -> Self {
        let option_count = config.options.set.len();
        let options: IndexMap<_, _> = config
            .options
            .set
            .into_iter()
            .map(|option| {
                let rendered = option.to_string();
                (option, rendered)
            })
            .collect();

        let initial = options
            .get_index_of(&config.options.initial)
            .expect("initial from config must be in the set");

        Self {
            options,
            selected: initial,
            cancellability: config.cancellability,
            base_config: config.base,
            first_row: 0,
            last_row: option_count - 1,
        }
    }

    pub fn raw_selection(&self) -> &O {
        let (option, _) = self
            .options
            .get_index(self.selected)
            .expect("internal menu state consistency");
        option
    }

    pub fn selection<'a>(
        &'a self,
    ) -> <C as SelectionCancellability<&'a O>>::Output {
        self.cancellability.select(self.raw_selection())
    }

    pub fn cancellability(&self) -> &C {
        &self.cancellability
    }

    pub fn cancellability_mut(&mut self) -> &mut C {
        &mut self.cancellability
    }

    pub fn select(&mut self, option: O) -> Result<(), UnknownOption<O>> {
        if let Some(index) = self.options.get_index_of(&option) {
            self.selected = index;
            Ok(())
        } else {
            Err(UnknownOption(option))
        }
    }

    pub fn on_tick(&mut self, tick: &mut Tick) -> Result<bool, CanvasError> {
        if tick.screen().needs_resize() {
            return Ok(true);
        }

        self.update_last_row(tick.screen().canvas_size());
        self.render(&mut *tick)?;

        while let Some(event) = tick.next_event() {
            match event {
                Event::Key(keys) => match keys {
                    KeyEvent {
                        main_key: Key::Esc,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => {
                        if self.cancellability.cancel_state().is_some() {
                            self.cancellability.set_cancel_state(true);
                            return Ok(false);
                        }
                    },

                    KeyEvent {
                        main_key: Key::Up,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_up(),

                    KeyEvent {
                        main_key: Key::Down,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_down(),

                    KeyEvent {
                        main_key: Key::Left,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_left(),

                    KeyEvent {
                        main_key: Key::Right,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_right(),

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

    /// Should be triggered when UP key is pressed.
    fn key_up(&mut self) {
        if self.is_cancelling() {
            self.cancellability.set_cancel_state(false);
        } else if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.first_row {
                self.first_row -= 1;
            }
        }
    }

    /// Should be triggered when DOWN key is pressed.
    fn key_down(&mut self) {
        if self.selected + 1 < self.options.len() {
            self.selected += 1;
            if self.selected >= self.last_row {
                self.first_row += 1;
            }
        } else if self.is_not_cancelling() {
            self.cancellability.set_cancel_state(true);
        }
    }

    /// Should be triggered when LEFT key is pressed.
    fn key_left(&mut self) {
        if self.is_not_cancelling() {
            self.cancellability.set_cancel_state(true);
        }
    }

    /// Should be triggered when RIGHT key is pressed.
    fn key_right(&mut self) {
        if self.is_cancelling() {
            self.cancellability.set_cancel_state(false);
        }
    }

    /// Returns if the selection is currently selecting the cancel option.
    fn is_cancelling(&self) -> bool {
        self.cancellability.cancel_state() == Some(true)
    }

    /// Returns if the selection is currently not selecting the cancel option
    /// AND the cancel option is enabled.
    fn is_not_cancelling(&self) -> bool {
        self.cancellability.cancel_state() == Some(false)
    }

    /// Updates the last row field from the computed end of the screen.
    fn update_last_row(&mut self, canvas_size: CoordPair) {
        self.last_row = self.screen_end(canvas_size);
    }

    /// Returns the index of the last visible option in the screen.
    fn screen_end(&self, canvas_size: CoordPair) -> usize {
        let cancel =
            if self.cancellability.cancel_state().is_some() { 4 } else { 0 };
        let mut available = canvas_size.y - self.base_config.title_y;
        available -= 2 * (self.base_config.pad_after_title - 1) + cancel;
        let extra = available / (self.base_config.pad_after_option + 1) - 2;
        self.first_row + usize::from(extra)
    }

    /// Returns the range of the visible options in the screen.
    fn range_of_screen(&self, canvas_size: CoordPair) -> Range<usize> {
        self.first_row .. self.screen_end(canvas_size)
    }

    /// Renders the whole menu.
    fn render(&self, tick: &mut Tick) -> Result<(), CanvasError> {
        tick.screen_mut().clear_canvas(self.base_config.background)?;
        self.render_title(&mut *tick)?;

        let arrow_style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.base_config.arrow_colors);

        let mut range = self.range_of_screen(tick.screen().canvas_size());
        self.render_up_arrow(&mut *tick, &arrow_style)?;
        self.render_down_arrow(&mut *tick, &arrow_style, &mut range)?;

        self.render_options(&mut *tick, range)?;
        let canvas_size = tick.screen().canvas_size();
        self.render_cancel(&mut *tick, canvas_size.y)?;

        Ok(())
    }

    /// Renders the title of the menu.
    fn render_title(&self, tick: &mut Tick) -> Result<(), CanvasError> {
        let title_style = TextStyle::default()
            .with_align(1, 2)
            .with_top_margin(self.base_config.title_y)
            .with_colors(self.base_config.title_colors)
            .with_max_height(
                self.base_config.pad_after_title.saturating_add(1),
            );
        tick.screen_mut().styled_text(&self.base_config.title, &title_style)?;
        Ok(())
    }

    /// Renders the UP arrow.
    fn render_up_arrow(
        &self,
        tick: &mut Tick,
        style: &TextStyle,
    ) -> Result<(), CanvasError> {
        if self.first_row > 0 {
            let mut option_y = self.y_of_option(self.first_row);
            option_y -= self.base_config.pad_after_option + 1;
            let style = style.with_top_margin(option_y);
            tick.screen_mut().styled_text("Ʌ", &style)?;
        }
        Ok(())
    }

    /// Renders the DOWN arrow and updates the given range of the screen.
    fn render_down_arrow(
        &self,
        tick: &mut Tick,
        style: &TextStyle,
        range: &mut Range<usize>,
    ) -> Result<(), CanvasError> {
        if range.end < self.options.len() {
            let option_y = self.y_of_option(range.end);
            let style = style.with_top_margin(option_y);
            tick.screen_mut().styled_text("V", &style)?;
        } else {
            range.end = self.options.len();
        }

        Ok(())
    }

    /// Renders all the options of the given range.
    fn render_options(
        &self,
        tick: &mut Tick,
        range: Range<usize>,
    ) -> Result<(), CanvasError> {
        for (i, (_, rendered)) in self.options[range.clone()].iter().enumerate()
        {
            let is_selected =
                range.start + i == self.selected && !self.is_cancelling();

            self.render_option(
                &mut *tick,
                rendered,
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
        option: &str,
        option_y: Coord,
        selected: bool,
    ) -> Result<(), CanvasError> {
        let mut buf = option.to_owned();
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
            self.base_config.selected_colors
        } else {
            self.base_config.unselected_colors
        };
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(colors)
            .with_top_margin(option_y);
        tick.screen_mut().styled_text(&buf, &style)?;

        Ok(())
    }

    /// Renders the cancel option, if any.
    fn render_cancel(
        &self,
        tick: &mut Tick,
        cancel_y: Coord,
    ) -> Result<(), CanvasError> {
        if let Some(selected) = self.cancellability.cancel_state() {
            let colors = if selected {
                self.base_config.selected_colors
            } else {
                self.base_config.unselected_colors
            };

            let style = TextStyle::default()
                .with_align(1, 3)
                .with_colors(colors)
                .with_top_margin(cancel_y - 2);
            let label_string =
                format!("> {} <", self.cancellability.cancel_label());
            tick.screen_mut().styled_text(&label_string, &style)?;
        }

        Ok(())
    }

    /// Gets the height of a given option (by index).
    fn y_of_option(&self, option: usize) -> Coord {
        let count = (option - self.first_row) as Coord;
        let before = (count + 1) * (self.base_config.pad_after_option + 1);
        before + self.base_config.pad_after_title + 1 + self.base_config.title_y
    }
}

/// An item of a prompt about a dangerous action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DangerPromptOption {
    /// Returned when user cancels this action.
    Cancel,
    /// Returned when user confirms this action.
    Ok,
}

impl fmt::Display for DangerPromptOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancel => write!(f, "CANCEL"),
            Self::Ok => write!(f, "OK"),
        }
    }
}

impl OptionItem for DangerPromptOption {}
