use std::sync::Arc;

use thedes_tui_core::{
    color::{Color, ColorPair},
    geometry::Coord,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    background: Color,
    title_colors: ColorPair,
    top_arrow_colors: ColorPair,
    selected_colors: ColorPair,
    unselected_colors: ColorPair,
    bottom_arrow_colors: ColorPair,
    left_margin: Coord,
    right_margin: Coord,
    top_margin: Coord,
    title_top_arrow_padding: Coord,
    top_arrow_items_padding: Coord,
    item_between_padding: Coord,
    items_bottom_arrow_padding: Coord,
    bottom_arrow_cancel_padding: Coord,
    bottom_margin: Coord,
    top_arrow: Arc<str>,
    bottom_arrow: Arc<str>,
    selected_left: Arc<str>,
    selected_right: Arc<str>,
    cancel_message: Arc<str>,
}

impl Default for Style {
    fn default() -> Self {
        let default_colors = ColorPair::default();
        Self {
            background: Color::default(),
            title_colors: ColorPair::default(),
            top_arrow_colors: ColorPair::default(),
            unselected_colors: default_colors,
            selected_colors: ColorPair {
                background: default_colors.foreground,
                foreground: default_colors.background,
            },
            bottom_arrow_colors: ColorPair::default(),
            left_margin: 1,
            right_margin: 1,
            top_margin: 1,
            title_top_arrow_padding: 1,
            top_arrow_items_padding: 1,
            item_between_padding: 1,
            items_bottom_arrow_padding: 1,
            bottom_arrow_cancel_padding: 1,
            bottom_margin: 1,
            top_arrow: Arc::from("⋀"),
            bottom_arrow: Arc::from("⋁"),
            selected_left: Arc::from("> "),
            selected_right: Arc::from(" <"),
            cancel_message: Arc::from("CANCEL"),
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

    pub fn with_top_arrow_colors(self, colors: ColorPair) -> Self {
        Self { top_arrow_colors: colors, ..self }
    }

    pub fn top_arrow_colors(&self) -> ColorPair {
        self.top_arrow_colors
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

    pub fn with_bottom_arrow_colors(self, colors: ColorPair) -> Self {
        Self { bottom_arrow_colors: colors, ..self }
    }

    pub fn bottom_arrow_colors(&self) -> ColorPair {
        self.bottom_arrow_colors
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

    pub fn with_title_top_arrow_padding(self, amount: Coord) -> Self {
        Self { title_top_arrow_padding: amount, ..self }
    }

    pub fn title_top_arrow_padding(&self) -> Coord {
        self.title_top_arrow_padding
    }

    pub fn with_top_arrow_items_padding(self, amount: Coord) -> Self {
        Self { top_arrow_items_padding: amount, ..self }
    }

    pub fn top_arrow_items_padding(&self) -> Coord {
        self.top_arrow_items_padding
    }

    pub fn with_item_between_padding(self, amount: Coord) -> Self {
        Self { item_between_padding: amount, ..self }
    }

    pub fn item_between_padding(&self) -> Coord {
        self.item_between_padding
    }

    pub fn with_items_bottom_arrow_padding(self, amount: Coord) -> Self {
        Self { items_bottom_arrow_padding: amount, ..self }
    }

    pub fn items_bottom_arrow_padding(&self) -> Coord {
        self.items_bottom_arrow_padding
    }

    pub fn with_bottom_arrow_cancel_padding(self, amount: Coord) -> Self {
        Self { bottom_arrow_cancel_padding: amount, ..self }
    }

    pub fn bottom_arrow_cancel_padding(&self) -> Coord {
        self.bottom_arrow_cancel_padding
    }

    pub fn with_bottom_margin(self, amount: Coord) -> Self {
        Self { bottom_margin: amount, ..self }
    }

    pub fn bottom_margin(&self) -> Coord {
        self.bottom_margin
    }

    pub fn with_top_arrow(self, text: impl AsRef<str>) -> Self {
        Self { top_arrow: text.as_ref().into(), ..self }
    }

    pub fn top_arrow(&self) -> &str {
        &self.top_arrow[..]
    }

    pub fn with_bottom_arrow(self, text: impl AsRef<str>) -> Self {
        Self { bottom_arrow: text.as_ref().into(), ..self }
    }

    pub fn bottom_arrow(&self) -> &str {
        &self.bottom_arrow[..]
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

    pub fn with_cancel_message(self, text: impl AsRef<str>) -> Self {
        Self { cancel_message: text.as_ref().into(), ..self }
    }

    pub fn cancel_message(&self) -> &str {
        &self.cancel_message[..]
    }
}
