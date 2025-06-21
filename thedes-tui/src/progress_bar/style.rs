use thedes_tui_core::{
    color::{BasicColor, Color, ColorPair},
    geometry::Coord,
};

#[derive(Debug, Clone)]
pub struct Style {
    title_y: Coord,
    title_colors: ColorPair,
    pad_after_title: Coord,
    bar_size: Coord,
    bar_colors: ColorPair,
    pad_after_bar: Coord,
    perc_colors: ColorPair,
    pad_after_perc: Coord,
    absolute_colors: ColorPair,
    pad_after_abs: Coord,
    status_colors: ColorPair,
    background: Color,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            title_y: 1,
            title_colors: ColorPair::default(),
            pad_after_title: 2,
            pad_after_bar: 0,
            pad_after_perc: 1,
            pad_after_abs: 1,
            bar_size: 32,
            bar_colors: ColorPair {
                foreground: BasicColor::White.into(),
                background: BasicColor::DarkGray.into(),
            },
            absolute_colors: ColorPair::default(),
            perc_colors: ColorPair::default(),
            status_colors: ColorPair::default(),
            background: BasicColor::Black.into(),
        }
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title_y(self, value: Coord) -> Self {
        Self { title_y: value, ..self }
    }

    pub fn title_y(&self) -> Coord {
        self.title_y
    }

    pub fn with_title_colors(self, value: ColorPair) -> Self {
        Self { title_colors: value, ..self }
    }

    pub fn title_colors(&self) -> ColorPair {
        self.title_colors
    }

    pub fn with_pad_after_title(self, value: Coord) -> Self {
        Self { pad_after_title: value, ..self }
    }

    pub fn pad_after_title(&self) -> Coord {
        self.pad_after_title
    }

    pub fn with_bar_size(self, value: Coord) -> Self {
        Self { bar_size: value, ..self }
    }

    pub fn bar_size(&self) -> Coord {
        self.bar_size
    }

    pub fn with_bar_colors(self, value: ColorPair) -> Self {
        Self { bar_colors: value, ..self }
    }

    pub fn bar_colors(&self) -> ColorPair {
        self.bar_colors
    }

    pub fn with_pad_after_bar(self, value: Coord) -> Self {
        Self { pad_after_bar: value, ..self }
    }

    pub fn pad_after_bar(&self) -> Coord {
        self.pad_after_bar
    }

    pub fn with_perc_colors(self, value: ColorPair) -> Self {
        Self { perc_colors: value, ..self }
    }

    pub fn perc_colors(&self) -> ColorPair {
        self.perc_colors
    }

    pub fn with_pad_after_perc(self, value: Coord) -> Self {
        Self { pad_after_perc: value, ..self }
    }

    pub fn pad_after_perc(&self) -> Coord {
        self.pad_after_perc
    }

    pub fn with_absolute_colors(self, value: ColorPair) -> Self {
        Self { absolute_colors: value, ..self }
    }

    pub fn absolute_colors(&self) -> ColorPair {
        self.absolute_colors
    }

    pub fn with_pad_after_abs(self, value: Coord) -> Self {
        Self { pad_after_abs: value, ..self }
    }

    pub fn pad_after_abs(&self) -> Coord {
        self.pad_after_abs
    }

    pub fn with_status_colors(self, value: ColorPair) -> Self {
        Self { status_colors: value, ..self }
    }

    pub fn status_colors(&self) -> ColorPair {
        self.status_colors
    }

    pub fn with_background(self, value: Color) -> Self {
        Self { background: value, ..self }
    }

    pub fn background(&self) -> Color {
        self.background
    }
}
