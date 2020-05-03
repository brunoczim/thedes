use crate::{
    error::Result,
    graphics::{
        Color,
        Color2,
        ColoredGString,
        ColorsKind,
        Grapheme,
        Style,
        Tile,
    },
    input::{Event, Key, KeyEvent},
    math::plane::{Coord2, Nat},
    terminal,
    ui::{LabeledOption, Labels},
};
use std::{future::Future, ops::Range};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Menu<O>
where
    O: LabeledOption,
{
    /// The title shown above the menu.
    pub title: ColoredGString<ColorsKind>,
    /// A list of options.
    pub options: Vec<O>,
    /// Colors for the arrows.
    pub arrow_colors: ColorsKind,
    /// Number of lines padded before the title.
    pub title_y: Nat,
    /// Number of lines padded after the title.
    pub pad_after_title: Nat,
    /// Number of lines padded after an option.
    pub pad_after_option: Nat,
}

impl<O> Menu<O>
where
    O: LabeledOption,
{
    /// Creates a new menu with default styles.
    pub fn new(title: ColoredGString<ColorsKind>, options: Vec<O>) -> Self {
        Self {
            title,
            options,
            arrow_colors: ColorsKind::from(Color2::default()),
            title_y: 1,
            pad_after_title: 2,
            pad_after_option: 1,
        }
    }

    /// Asks for the user to select an item of the menu without cancel option.
    pub async fn select<F, A>(
        &self,
        term: &terminal::Handle,
        render_bg: F,
    ) -> Result<usize>
    where
        F: FnMut(&mut terminal::Screen) -> A,
        A: Future<Output = Result<()>>,
    {
        let mut selected = 0;
        let mut start = 0;

        self.render(term, start, Some(selected), None, &mut render_bg).await?;
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
                        self.render(
                            term,
                            start,
                            Some(selected),
                            None,
                            &mut render_bg,
                        )
                        .await?;
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
                        self.render(
                            term,
                            start,
                            Some(selected),
                            None,
                            &mut render_bg,
                        )
                        .await?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    alt: false,
                    ctrl: false,
                    shift: false,
                }) => break,

                Event::Resize(evt) => {
                    self.render(
                        term,
                        start,
                        Some(selected),
                        None,
                        &mut render_bg,
                    )
                    .await?;
                    last_row = self.screen_end(start, evt.size, false);
                },

                _ => (),
            }
        }

        Ok(selected)
    }

    /// Asks for the user to select an item of the menu with a cancel option.
    pub async fn select_with_cancel<F, A>(
        &self,
        term: &terminal::Handle,
        cancel: &Labels,
        render_bg: F,
    ) -> Result<Option<usize>>
    where
        F: FnMut(&mut terminal::Screen) -> A,
        A: Future<Output = Result<()>>,
    {
        let mut selected = 0;
        let mut is_cancel = self.options.len() == 0;
        let mut start = 0;

        self.render(
            term,
            start,
            Some(selected).filter(|_| !is_cancel),
            Some(cancel),
            &mut render_bg,
        )
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
                        self.render(
                            term,
                            start,
                            Some(selected),
                            Some(cancel),
                            &mut render_bg,
                        )
                        .await?;
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
                            Some(cancel),
                            &mut render_bg,
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
                            Some(cancel),
                            &mut render_bg,
                        )
                        .await?;
                    } else if !is_cancel {
                        is_cancel = true;
                        self.render(
                            term,
                            start,
                            None,
                            Some(cancel),
                            &mut render_bg,
                        )
                        .await?;
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
                        self.render(
                            term,
                            start,
                            None,
                            Some(cancel),
                            &mut render_bg,
                        )
                        .await?;
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
                        self.render(
                            term,
                            start,
                            Some(selected),
                            Some(cancel),
                            &mut render_bg,
                        )
                        .await?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => break if is_cancel { None } else { Some(selected) },

                Event::Resize(evt) => {
                    self.render(
                        term,
                        start,
                        Some(selected),
                        Some(cancel),
                        &mut render_bg,
                    )
                    .await?;
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

    async fn render<F, A>(
        &self,
        term: &terminal::Handle,
        start: usize,
        selected: Option<usize>,
        cancel: Option<Labels>,
        render_bg: &mut F,
    ) -> Result<()>
    where
        F: FnMut(&mut terminal::Screen) -> A,
        A: Future<Output = Result<()>>,
    {
        let screen_size = term.screen_size();

        let mut screen = term.lock_screen().await;
        render_bg(&mut screen).await?;
        let style = Style::new()
            .align(1, 2)
            .top_margin(self.title_y)
            .max_height(self.pad_after_title.saturating_add(1));
        screen.styled_text(&self.title, style)?;

        let mut range =
            self.range_of_screen(start, term.screen_size(), cancel.is_some());
        if start > 0 {
            let y = self.y_of_option(start, start) - self.pad_after_option - 1;
            let style = Style::new().align(1, 2).top_margin(y);
            screen.styled_text(
                &colored_gstring![(gstring!["Ʌ"], self.arrow_colors.clone())],
                style,
            )?
        }
        if range.end < self.options.len() {
            let y = self.y_of_option(start, range.end);
            let style = Style::new().align(1, 2).top_margin(y);
            screen.styled_text(
                &colored_gstring![(gstring!["V"], self.arrow_colors.clone())],
                style,
            )?
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

        if let Some(cancel) = cancel {
            self.render_cancel(
                &mut screen,
                cancel,
                screen_size.y,
                selected.is_none(),
            )?;
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
        let screen_size = screen.handle().screen_size();
        let label = option.label(selected).wrap_with(
            screen_size.x,
            gstring!["> "],
            gstring![" <"],
        );
        let style = Style::new().align(1, 2).top_margin(y);
        screen.styled_text(&label, style)?;
        Ok(())
    }

    fn render_cancel(
        &self,
        screen: &mut terminal::Screen,
        cancel_y: Nat,
        selected: bool,
        cancel: Labels,
    ) -> Result<()> {
        let screen_size = screen.handle().screen_size();
        let label = cancel.label(selected).wrap_with(
            screen_size.x,
            gstring!["> "],
            gstring![" <"],
        );
        let style = Style::new().align(1, 3).top_margin(cancel_y - 2);
        screen.styled_text(&label, style)?;
        Ok(())
    }
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

impl LabeledOption for DangerPromptOption {
    fn label(&self, selected: bool) -> ColoredGString<ColorsKind> {
        let string = match self {
            DangerPromptOption::Cancel => "CANCEL",
            DangerPromptOption::Ok => "OK",
        };

        let colors = if selected {
            ColorsKind::default()
        } else {
            !ColorsKind::default()
        };

        colored_gstring![(gstring![string], colors)]
    }
}
