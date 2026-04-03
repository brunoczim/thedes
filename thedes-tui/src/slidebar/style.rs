use std::sync::Arc;

use thedes_tui_core::{
    color::{BasicColor, Color, ColorPair},
    geometry::Coord,
    grapheme,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    background: Color,
    title_colors: ColorPair,
    message_colors: ColorPair,
    bar_colors: ColorPair,
    back_colors: ColorPair,
    left_margin: Coord,
    right_margin: Coord,
    top_margin: Coord,
    title_slidebar_padding: Coord,
    slidebar_message_padding: Coord,
    message_back_padding: Coord,
    bottom_margin: Coord,
    left_arrow: Arc<str>,
    right_arrow: Arc<str>,
    left_bar_ch: grapheme::Id,
    right_bar_ch: grapheme::Id,
    slide_handle: Arc<str>,
    back_label: Arc<str>,
}

impl Default for Style {
    fn default() -> Self {
        let default_colors = ColorPair::default();
        Self {
            background: Color::default(),
            title_colors: default_colors,
            message_colors: default_colors,
            bar_colors: default_colors,
            back_colors: ColorPair {
                background: BasicColor::White.into(),
                foreground: BasicColor::Black.into(),
            },
            left_margin: 1,
            right_margin: 1,
            top_margin: 1,
            title_slidebar_padding: 1,
            slidebar_message_padding: 1,
            message_back_padding: 1,
            bottom_margin: 1,
            left_arrow: Arc::from(""),
            right_arrow: Arc::from(""),
            left_bar_ch: '═'.into(),
            right_bar_ch: '─'.into(),
            slide_handle: Arc::from("❚"),
            back_label: Arc::from("BACK"),
        }
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background(&self) -> Color {
        self.background
    }

    pub fn with_background(mut self, value: Color) -> Self {
        self.background = value;
        self
    }

    pub fn title_colors(&self) -> ColorPair {
        self.title_colors
    }

    pub fn with_title_colors(mut self, value: ColorPair) -> Self {
        self.title_colors = value;
        self
    }

    pub fn message_colors(&self) -> ColorPair {
        self.message_colors
    }

    pub fn with_message_colors(mut self, value: ColorPair) -> Self {
        self.message_colors = value;
        self
    }

    pub fn bar_colors(&self) -> ColorPair {
        self.bar_colors
    }

    pub fn with_bar_colors(mut self, value: ColorPair) -> Self {
        self.bar_colors = value;
        self
    }

    pub fn back_colors(&self) -> ColorPair {
        self.back_colors
    }

    pub fn with_back_colors(mut self, value: ColorPair) -> Self {
        self.back_colors = value;
        self
    }

    pub fn left_margin(&self) -> Coord {
        self.left_margin
    }

    pub fn with_left_margin(mut self, value: Coord) -> Self {
        self.left_margin = value;
        self
    }

    pub fn right_margin(&self) -> Coord {
        self.right_margin
    }

    pub fn with_right_margin(mut self, value: Coord) -> Self {
        self.right_margin = value;
        self
    }

    pub fn top_margin(&self) -> Coord {
        self.top_margin
    }

    pub fn with_top_margin(mut self, value: Coord) -> Self {
        self.top_margin = value;
        self
    }

    pub fn title_slidebar_padding(&self) -> Coord {
        self.title_slidebar_padding
    }

    pub fn with_title_slidebar_padding(mut self, value: Coord) -> Self {
        self.title_slidebar_padding = value;
        self
    }

    pub fn slidebar_message_padding(&self) -> Coord {
        self.slidebar_message_padding
    }

    pub fn with_slidebar_message_padding(mut self, value: Coord) -> Self {
        self.slidebar_message_padding = value;
        self
    }

    pub fn message_back_padding(&self) -> Coord {
        self.message_back_padding
    }

    pub fn with_message_back_padding(mut self, value: Coord) -> Self {
        self.message_back_padding = value;
        self
    }

    pub fn bottom_margin(&self) -> Coord {
        self.bottom_margin
    }

    pub fn with_bottom_margin(mut self, value: Coord) -> Self {
        self.bottom_margin = value;
        self
    }

    pub fn left_arrow(&self) -> &str {
        &self.left_arrow[..]
    }

    pub fn with_left_arrow(mut self, value: impl AsRef<str>) -> Self {
        self.left_arrow = value.as_ref().into();
        self
    }

    pub fn right_arrow(&self) -> &str {
        &self.right_arrow[..]
    }

    pub fn with_right_arrow(mut self, value: impl AsRef<str>) -> Self {
        self.right_arrow = value.as_ref().into();
        self
    }

    pub fn left_bar_ch(&self) -> grapheme::Id {
        self.left_bar_ch
    }

    pub fn with_left_bar_ch(mut self, value: grapheme::Id) -> Self {
        self.left_bar_ch = value;
        self
    }

    pub fn right_bar_ch(&self) -> grapheme::Id {
        self.right_bar_ch
    }

    pub fn with_right_bar_ch(mut self, value: grapheme::Id) -> Self {
        self.right_bar_ch = value;
        self
    }

    pub fn slide_handle(&self) -> &str {
        &self.slide_handle[..]
    }

    pub fn with_slide_handle(mut self, value: impl AsRef<str>) -> Self {
        self.slide_handle = value.as_ref().into();
        self
    }

    pub fn back_label(&self) -> &str {
        &self.back_label[..]
    }

    pub fn with_back_label(mut self, value: impl AsRef<str>) -> Self {
        self.back_label = value.as_ref().into();
        self
    }
}
