use std::sync::Arc;

use thedes_tui_core::{
    color::{Color, ColorPair},
    geometry::Coord,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    background: Color,
    title_colors: ColorPair,
    message_colors: ColorPair,
    selected_colors: ColorPair,
    unselected_colors: ColorPair,
    left_margin: Coord,
    right_margin: Coord,
    top_margin: Coord,
    title_message_padding: Coord,
    message_ok_padding: Coord,
    ok_cancel_padding: Coord,
    bottom_margin: Coord,
    selected_left: Arc<str>,
    selected_right: Arc<str>,
    ok_label: Arc<str>,
    cancel_label: Arc<str>,
}

impl Default for Style {
    fn default() -> Self {
        let default_colors = ColorPair::default();
        let inverted_colors = ColorPair {
            background: default_colors.foreground,
            foreground: default_colors.background,
        };
        Self {
            background: Color::default(),
            title_colors: default_colors,
            message_colors: default_colors,
            unselected_colors: default_colors,
            selected_colors: inverted_colors,
            left_margin: 1,
            right_margin: 1,
            top_margin: 1,
            bottom_margin: 1,
            title_message_padding: 1,
            message_ok_padding: 1,
            ok_cancel_padding: 1,
            selected_left: Arc::from("> "),
            selected_right: Arc::from(" <"),
            ok_label: Arc::from("OK"),
            cancel_label: Arc::from("CANCEL"),
        }
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_background(self, color: Color) -> Self {
        Self { background: color, ..self }
    }

    pub fn background(&self) -> Color {
        self.background
    }

    pub fn with_title_colors(self, colors: ColorPair) -> Self {
        Self { title_colors: colors, ..self }
    }

    pub fn title_colors(&self) -> ColorPair {
        self.title_colors
    }

    pub fn with_selected_colors(self, colors: ColorPair) -> Self {
        Self { selected_colors: colors, ..self }
    }

    pub fn selected_colors(&self) -> ColorPair {
        self.selected_colors
    }

    pub fn with_unselected_colors(self, colors: ColorPair) -> Self {
        Self { unselected_colors: colors, ..self }
    }

    pub fn unselected_colors(&self) -> ColorPair {
        self.unselected_colors
    }

    pub fn with_message_colors(self, colors: ColorPair) -> Self {
        Self { message_colors: colors, ..self }
    }

    pub fn message_colors(&self) -> ColorPair {
        self.message_colors
    }

    pub fn with_left_margin(self, amount: Coord) -> Self {
        Self { left_margin: amount, ..self }
    }

    pub fn left_margin(&self) -> Coord {
        self.left_margin
    }

    pub fn with_right_margin(self, amount: Coord) -> Self {
        Self { right_margin: amount, ..self }
    }

    pub fn right_margin(&self) -> Coord {
        self.right_margin
    }

    pub fn with_top_margin(self, amount: Coord) -> Self {
        Self { top_margin: amount, ..self }
    }

    pub fn top_margin(&self) -> Coord {
        self.top_margin
    }

    pub fn with_title_message_padding(self, amount: Coord) -> Self {
        Self { title_message_padding: amount, ..self }
    }

    pub fn title_message_padding(&self) -> Coord {
        self.title_message_padding
    }

    pub fn with_message_ok_padding(self, amount: Coord) -> Self {
        Self { message_ok_padding: amount, ..self }
    }

    pub fn message_ok_padding(&self) -> Coord {
        self.message_ok_padding
    }

    pub fn with_ok_cancel_padding(self, amount: Coord) -> Self {
        Self { ok_cancel_padding: amount, ..self }
    }

    pub fn ok_cancel_padding(&self) -> Coord {
        self.ok_cancel_padding
    }

    pub fn with_bottom_margin(self, amount: Coord) -> Self {
        Self { bottom_margin: amount, ..self }
    }

    pub fn bottom_margin(&self) -> Coord {
        self.bottom_margin
    }

    pub fn with_selected_left(self, text: impl AsRef<str>) -> Self {
        Self { selected_left: text.as_ref().into(), ..self }
    }

    pub fn selected_left(&self) -> &str {
        &self.selected_left[..]
    }

    pub fn with_selected_right(self, text: impl AsRef<str>) -> Self {
        Self { selected_right: text.as_ref().into(), ..self }
    }

    pub fn selected_right(&self) -> &str {
        &self.selected_right[..]
    }

    pub fn with_ok_label(self, text: impl AsRef<str>) -> Self {
        Self { ok_label: text.as_ref().into(), ..self }
    }

    pub fn ok_label(&self) -> &str {
        &self.ok_label[..]
    }

    pub fn with_cancel_label(self, text: impl AsRef<str>) -> Self {
        Self { cancel_label: text.as_ref().into(), ..self }
    }

    pub fn cancel_label(&self) -> &str {
        &self.cancel_label[..]
    }
}
