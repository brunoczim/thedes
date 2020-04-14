use crate::{
    coord::Nat,
    error::Result,
    graphics::{Color, Color2, GString, Style},
    input::{Event, Key, KeyEvent},
    terminal,
};

/// An info dialog, with just an Ok option.
#[derive(Debug, Clone)]
pub struct InfoDialog {
    /// Title to be shown.
    pub title: GString,
    /// Long text message to be shown.
    pub message: GString,
    /// Settings such as margin and alignment.
    pub style: Style,
    /// Colors shown with the title.
    pub title_colors: Color2,
    /// Colors shown with the selected option.
    pub selected_colors: Color2,
    /// Position of the title in height.
    pub title_y: Nat,
    /// Color of the background.
    pub bg: Color,
}

impl InfoDialog {
    /// Creates a dialog with default style settings.
    pub fn new(title: GString, message: GString) -> Self {
        Self {
            title,
            message,
            style: Style::new()
                .align(1, 2)
                .colors(Color2::default())
                .top_margin(4)
                .bottom_margin(2),
            title_colors: Color2::default(),
            selected_colors: !Color2::default(),
            title_y: 1,
            bg: Color::Black,
        }
    }

    /// Runs this dialog showing it to the user, awaiting OK!
    pub async fn run(&self, term: &terminal::Handle) -> Result<()> {
        self.render(term).await?;

        loop {
            match term.listen_event().await? {
                Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => break Ok(()),

                Event::Key(KeyEvent {
                    main_key: Key::Esc,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => break Ok(()),

                Event::Resize(_) => self.render(term).await?,

                _ => (),
            }
        }
    }

    async fn render(&self, term: &terminal::Handle) -> Result<()> {
        let mut screen = term.lock_screen().await;
        screen.clear(self.bg);
        let style = Style::new()
            .align(1, 2)
            .colors(self.title_colors)
            .top_margin(self.title_y);
        screen.styled_text(&self.title, style)?;

        let pos = screen.styled_text(&self.message, self.style)?;
        tracing::debug!(?pos);
        let ok_string = gstring!["> OK <"];
        let style = Style::new()
            .align(1, 2)
            .colors(self.selected_colors)
            .top_margin(pos + 2);
        screen.styled_text(&ok_string, style)?;
        Ok(())
    }
}
