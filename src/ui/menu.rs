use crate::{
    coord::{Coord2, Nat},
    error::Result,
    graphics::{Color, Color2, GString, Grapheme, Style},
    input::{Event, Key, KeyEvent},
    terminal,
};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Menu<O>
where
    O: MenuOption,
{
    /// The title shown above the menu.
    pub title: GString,
    /// A list of options.
    pub options: Vec<O>,
    /// Colors for the title.
    pub title_colors: Color2,
    /// Colors for the arrows.
    pub arrow_colors: Color2,
    /// Colors for selected options.
    pub selected_colors: Color2,
    /// Colors for unselected options.
    pub unselected_colors: Color2,
    /// Color of the background of no text.
    pub bg: Color,
    /// Number of lines padded before the title.
    pub title_y: Nat,
    /// Number of lines padded after the title.
    pub pad_after_title: Nat,
    /// Number of lines padded after an option.
    pub pad_after_option: Nat,
}

impl<O> Menu<O>
where
    O: MenuOption,
{
    /// Creates a new menu with default styles.
    pub fn new(title: GString, options: Vec<O>) -> Self {
        Self {
            title,
            options,
            title_colors: Color2::default(),
            arrow_colors: Color2::default(),
            selected_colors: !Color2::default(),
            unselected_colors: Color2::default(),
            bg: Color::Black,
            title_y: 1,
            pad_after_title: 2,
            pad_after_option: 1,
        }
    }

    /// Asks for the user to select an item of the menu without cancel option.
    pub async fn select(&self, term: &terminal::Handle) -> Result<usize> {
        let mut selected = 0;
        let mut start = 0;

        self.render(term, start, Some(selected), false).await?;
        let mut last_row = self.screen_end(start, term.screen_size(), false);

        loop {
            match term.listen_event().await? {
                Event::Key(KeyEvent {
                    main_key: Key::Up,
                    alt: false,
                    ctrl: false,
                    shift: false,
                }) => {
                    if selected > 0 {
                        selected -= 1;
                        if selected < start {
                            start -= 1;
                            last_row = self.screen_end(
                                start,
                                term.screen_size(),
                                false,
                            );
                        }
                        self.render(term, start, Some(selected), false).await?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Down,
                    alt: false,
                    ctrl: false,
                    shift: false,
                }) => {
                    if selected + 1 < self.options.len() {
                        selected += 1;
                        if selected >= last_row {
                            start += 1;
                            last_row = self.screen_end(
                                start,
                                term.screen_size(),
                                false,
                            );
                        }
                        self.render(term, start, Some(selected), false).await?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    alt: false,
                    ctrl: false,
                    shift: false,
                }) => break,

                Event::Resize(evt) => {
                    self.render(term, start, Some(selected), false).await?;
                    last_row = self.screen_end(start, evt.size, false);
                },

                _ => (),
            }
        }

        Ok(selected)
    }

    /// Asks for the user to select an item of the menu with a cancel option.
    pub async fn select_with_cancel(
        &self,
        term: &terminal::Handle,
    ) -> Result<Option<usize>> {
        let mut selected = 0;
        let mut is_cancel = self.options.len() == 0;
        let mut start = 0;

        self.render(term, start, Some(selected).filter(|_| !is_cancel), true)
            .await?;
        let mut last_row = self.screen_end(start, term.screen_size(), true);

        let ret = loop {
            match term.listen_event().await? {
                Event::Key(KeyEvent {
                    main_key: Key::Esc,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => break None,

                Event::Key(KeyEvent {
                    main_key: Key::Up,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if is_cancel && self.options.len() > 0 {
                        is_cancel = false;
                        self.render(term, start, Some(selected), true).await?;
                    } else if selected > 0 {
                        selected -= 1;
                        if selected < start {
                            start -= 1;
                            last_row = self.screen_end(
                                start,
                                term.screen_size(),
                                true,
                            );
                        }
                        self.render(
                            term,
                            start,
                            Some(selected).filter(|_| !is_cancel),
                            true,
                        )
                        .await?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if selected + 1 < self.options.len() {
                        selected += 1;
                        if selected >= last_row {
                            start += 1;
                            last_row = self.screen_end(
                                start,
                                term.screen_size(),
                                true,
                            );
                        }
                        self.render(
                            term,
                            start,
                            Some(selected).filter(|_| !is_cancel),
                            true,
                        )
                        .await?;
                    } else if !is_cancel {
                        is_cancel = true;
                        self.render(term, start, None, true).await?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Left,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if !is_cancel {
                        is_cancel = true;
                        self.render(term, start, None, true).await?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Right,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if is_cancel && self.options.len() > 0 {
                        is_cancel = false;
                        self.render(term, start, Some(selected), true).await?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => break if is_cancel { None } else { Some(selected) },

                Event::Resize(evt) => {
                    self.render(term, start, Some(selected), true).await?;
                    last_row = self.screen_end(start, evt.size, true);
                },

                _ => (),
            }
        };

        Ok(ret)
    }

    fn y_of_option(&self, start: usize, option: usize) -> Nat {
        let count = (option - start) as Nat;
        let before = (count + 1) * (self.pad_after_option + 1);
        before + self.pad_after_title + 1 + self.title_y
    }

    fn screen_end(
        &self,
        start: usize,
        screen_size: Coord2<Nat>,
        cancel: bool,
    ) -> usize {
        let cancel = if cancel { 4 } else { 0 };
        let available = screen_size.y - self.title_y;
        let available = available - 2 * (self.pad_after_title - 1) - cancel;
        let extra = available / (self.pad_after_option + 1) - 2;
        start + extra as usize
    }

    fn range_of_screen(
        &self,
        start: usize,
        screen_size: Coord2<Nat>,
        cancel: bool,
    ) -> Range<usize> {
        start .. self.screen_end(start, screen_size, cancel)
    }

    async fn render(
        &self,
        term: &terminal::Handle,
        start: usize,
        selected: Option<usize>,
        cancel: bool,
    ) -> Result<()> {
        let screen_size = term.screen_size();

        let mut screen = term.lock_screen().await;
        screen.clear(self.bg);
        let style = Style::new()
            .align(1, 2)
            .top_margin(self.title_y)
            .colors(self.title_colors)
            .max_height(self.pad_after_title.saturating_add(1));
        screen.styled_text(&self.title, style)?;

        let mut range = self.range_of_screen(start, term.screen_size(), cancel);
        if start > 0 {
            let y = self.y_of_option(start, start) - self.pad_after_option - 1;
            let style = Style::new()
                .align(1, 2)
                .colors(self.arrow_colors)
                .top_margin(y);
            screen.styled_text(&gstring!["Ʌ"], style)?;
        }
        if range.end < self.options.len() {
            let y = self.y_of_option(start, range.end);
            let style = Style::new()
                .align(1, 2)
                .colors(self.arrow_colors)
                .top_margin(y);
            screen.styled_text(&gstring!["V"], style)?;
        } else {
            range.end = self.options.len();
        }
        for (i, option) in self.options[range.clone()].iter().enumerate() {
            let is_selected = Some(range.start + i) == selected;
            self.render_option(
                &mut screen,
                option,
                self.y_of_option(start, range.start + i),
                is_selected,
            )?;
        }

        if cancel {
            self.render_cancel(&mut screen, screen_size.y, selected.is_none())?;
        }
        Ok(())
    }

    fn render_option(
        &self,
        screen: &mut terminal::Screen,
        option: &O,
        y: Nat,
        selected: bool,
    ) -> Result<()> {
        let mut buf = option.name();
        let mut len = buf.count_graphemes();
        let screen_size = screen.handle().screen_size();

        if len as Nat % 2 != screen_size.x % 2 {
            buf = gconcat![buf, Grapheme::space()];
            len += 1;
        }

        if screen_size.x - 4 < len as Nat {
            buf = gconcat![buf.index(.. len - 5), Grapheme::new_lossy("…")];
            #[allow(unused_assignments)]
            {
                len -= 4;
            }
        }

        buf = gconcat![gstring!["> "], buf, gstring![" <"]];

        let colors = if selected {
            self.selected_colors
        } else {
            self.unselected_colors
        };
        let style = Style::new().align(1, 2).colors(colors).top_margin(y);
        screen.styled_text(&buf, style)?;
        Ok(())
    }

    fn render_cancel(
        &self,
        screen: &mut terminal::Screen,
        cancel_y: Nat,
        selected: bool,
    ) -> Result<()> {
        let colors = if selected {
            self.selected_colors
        } else {
            self.unselected_colors
        };
        let string = gstring!["> Cancel <"];

        let style =
            Style::new().align(1, 3).colors(colors).top_margin(cancel_y - 2);
        screen.styled_text(&string, style)?;

        Ok(())
    }
}

/// A trait representing a menu option.
pub trait MenuOption {
    /// Returns the display name of this option.
    fn name(&self) -> GString;
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
    /// Creates a menu over a dangerous prompt.
    pub fn menu(title: GString) -> Menu<Self> {
        Menu::new(
            title,
            vec![DangerPromptOption::Ok, DangerPromptOption::Cancel],
        )
    }
}

impl MenuOption for DangerPromptOption {
    fn name(&self) -> GString {
        let string = match self {
            DangerPromptOption::Cancel => "CANCEL",
            DangerPromptOption::Ok => "OK",
        };

        gstring![string]
    }
}
