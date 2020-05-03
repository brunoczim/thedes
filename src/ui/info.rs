use crate::{
    error::Result,
    graphics::{Color, ColoredGString, ColorsKind, Style},
    input::{Event, Key, KeyEvent},
    math::plane::Nat,
    terminal,
    ui::Labels,
};
use std::future::Future;

/// An info dialog, with just an Ok option.
#[derive(Debug, Clone)]
pub struct InfoDialog {
    /// Title to be shown.
    pub title: ColoredGString<ColorsKind>,
    /// Long text message to be shown.
    pub message: ColoredGString<ColorsKind>,
    /// Ok label.
    pub ok: ColoredGString<ColorsKind>,
    /// Settings such as margin and alignment.
    pub style: Style,
    /// Colors shown with the title.
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
            style: Style::new().align(1, 2).top_margin(4).bottom_margin(2),
            ok: colored_gstring![(gstring!["OK"], ColorsKind::default())],
            title_y: 1,
            bg: Color::Black,
        }
    }

    /// Runs this dialog showing it to the user, awaiting OK!
    pub async fn run<F, A>(
        &self,
        term: &terminal::Handle,
        render_bg: F,
    ) -> Result<()>
    where
        F: FnMut(&mut terminal::Screen) -> A,
        A: Future<Output = Result<()>>,
    {
        self.render(term, &mut render_bg).await?;

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

                Event::Resize(_) => self.render(term, &mut render_bg).await?,

                _ => (),
            }
        }
    }

    async fn render<F, A>(
        &self,
        term: &terminal::Handle,
        render_bg: &mut F,
    ) -> Result<()>
    where
        F: FnMut(&mut terminal::Screen) -> A,
        A: Future<Output = Result<()>>,
    {
        let mut screen = term.lock_screen().await;
        let screen_size = screen.handle().screen_size();
        render_bg(&mut screen).await?;
        let style = Style::new().align(1, 2).top_margin(self.title_y);
        screen.styled_text(&self.title, style)?;

        let pos = screen.styled_text(&self.message, self.style)?;
        let style = Style::new().align(1, 2).top_margin(pos + 2);
        let label = self.ok.label(true).wrap_with(
            screen_size.x,
            gstring!["> "],
            gstring![" <"],
        );
        screen.styled_text(&label, style)?;
        Ok(())
    }
}
