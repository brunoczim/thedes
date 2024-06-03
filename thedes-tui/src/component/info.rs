//! An INFO dialong: just shows a message.

use crate::{
    color::{BasicColor, Color, ColorPair},
    event::{Event, Key},
    geometry::Coord,
    screen::TextStyle,
    RenderError,
    Tick,
};

/// An info dialog, with just an Ok option.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    title: String,
    message: String,
    ok_label: String,
    style: TextStyle,
    title_colors: ColorPair,
    selected_colors: ColorPair,
    title_y: Coord,
    background: Color,
}

impl Config {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: String::new(),
            ok_label: "OK".into(),
            style: TextStyle::default()
                .with_align(1, 2)
                .with_colors(ColorPair::default())
                .with_top_margin(4)
                .with_bottom_margin(2),
            title_colors: ColorPair::default(),
            selected_colors: !ColorPair::default(),
            title_y: 1,
            background: BasicColor::Black.into(),
        }
    }

    pub fn with_message(self, message: impl Into<String>) -> Self {
        Self { message: message.into(), ..self }
    }

    pub fn with_ok_label(self, label: impl Into<String>) -> Self {
        Self { ok_label: label.into(), ..self }
    }

    pub fn with_style(self, style: TextStyle) -> Self {
        Self { style, ..self }
    }

    pub fn with_title_colors(self, colors: ColorPair) -> Self {
        Self { title_colors: colors, ..self }
    }

    pub fn with_selected_colors(self, colors: ColorPair) -> Self {
        Self { selected_colors: colors, ..self }
    }

    pub fn with_title_y(self, y: Coord) -> Self {
        Self { title_y: y, ..self }
    }

    pub fn with_background(self, color: impl Into<Color>) -> Self {
        Self { background: color.into(), ..self }
    }
}

#[derive(Debug)]
pub struct InfoDialog {
    config: Config,
    initialized: bool,
}

impl InfoDialog {
    pub fn new(config: Config) -> Self {
        Self { config, initialized: false }
    }

    pub fn on_tick(&mut self, tick: &mut Tick) -> Result<bool, RenderError> {
        if tick.screen().needs_resize() {
            return Ok(true);
        }

        if !self.initialized {
            self.render(&mut *tick)?;
            self.initialized = true;
        }

        while let Some(event) = tick.next_event() {
            match event {
                Event::Key(key_evt) => match key_evt.main_key {
                    Key::Enter | Key::Esc => return Ok(false),
                    _ => (),
                },
                _ => (),
            }
        }

        Ok(true)
    }

    /// Renders the whole dialog.
    fn render(&self, tick: &mut Tick) -> Result<(), RenderError> {
        tick.screen_mut().clear_canvas(self.config.background)?;
        self.render_title(&mut *tick)?;
        let pos = self.render_message(&mut *tick)?;
        self.render_ok(&mut *tick, pos)?;
        Ok(())
    }

    /// Renders the title of the dialog.
    fn render_title(&self, tick: &mut Tick) -> Result<(), RenderError> {
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.config.title_colors)
            .with_top_margin(self.config.title_y);
        tick.screen_mut().print(&self.config.title, &style)?;
        Ok(())
    }

    /// Renders the message of the dialog.
    fn render_message(&self, tick: &mut Tick) -> Result<Coord, RenderError> {
        tick.screen_mut().print(&self.config.message, &self.config.style)
    }

    /// Renders the OK button.
    fn render_ok(
        &self,
        tick: &mut Tick,
        pos: Coord,
    ) -> Result<(), RenderError> {
        let style = TextStyle::default()
            .with_align(1, 2)
            .with_colors(self.config.selected_colors)
            .with_top_margin(pos + 2);
        let label_string = format!("> {} <", &self.config.ok_label);
        tick.screen_mut().print(&label_string, &style)?;
        Ok(())
    }
}
