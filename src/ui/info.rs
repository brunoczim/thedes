use crate::{
    coord::Nat,
    error::Result,
    graphics::{Color, Color2, Grapheme, Style},
    input::{Event, Key, KeyEvent},
    terminal,
};

/// An info dialog, with just an Ok option.
#[derive(Debug, Copy, Clone)]
pub struct InfoDialog<'title, 'msg> {
    /// Title to be shown.
    pub title: &'title [Grapheme],
    /// Long text message to be shown.
    pub message: &'msg [Grapheme],
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

impl<'title, 'msg> InfoDialog<'title, 'msg> {
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
        screen.clear_screen(self.bg);
        let style = Style::new()
            .align(1, 2)
            .colors(self.title_colors)
            .top_margin(self.title_y);
        screen.styled_text(self.title, style)?;

        let pos = screen.styled_text(self.message, self.style)?;
        let ok_string = Grapheme::expect_iter("> OK <").collect::<Vec<_>>();
        let style = Style::new()
            .align(1, 2)
            .colors(self.selected_colors)
            .top_margin(pos + 2);
        screen.styled_text(ok_string, style)?;
        Ok(())
    }
}
