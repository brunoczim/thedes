use crate::{
    coord::Nat,
    error::Result,
    graphics::{Color, Color2, GString, Style},
    input::{Event, Key, KeyEvent},
    terminal,
};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputDialogItem {
    Ok,
    Cancel,
}

/// A dialog asking for user input, possibly filtered.
pub struct InputDialog<'term, F>
where
    F: FnMut(char) -> bool,
{
    title: GString,
    buffer: String,
    term: &'term terminal::Handle,
    max: Nat,
    filter: F,
    /// Colors of the title.
    pub title_colors: Color2,
    /// Selected option colors.
    pub selected_colors: Color2,
    /// Unselected option colors.
    pub unselected_colors: Color2,
    /// Input box's cursor colors.
    pub cursor_colors: Color2,
    /// Input box colors.
    pub box_colors: Color2,
    /// Background of non-text areas.
    pub bg: Color,
    /// Position of the title.
    pub title_y: Nat,
    /// Padding lines inserted after the title.
    pub pad_after_title: Nat,
    /// Padding lines inserted after the box.
    pub pad_after_box: Nat,
    /// Padding lines inserted after the OK option.
    pub pad_after_ok: Nat,
}

impl<'term, F> InputDialog<'term, F>
where
    F: FnMut(char) -> bool,
{
    /// Creates a new input dialog, with the given title, initial buffer,
    /// maximum input size, and filter function.
    pub fn new(
        title: GString,
        buffer: String,
        term: &'term terminal::Handle,
        max: Nat,
        filter: F,
    ) -> Self {
        Self {
            title,
            buffer,
            term,
            filter,
            max: max.min(term.min_screen().x - 1),
            title_colors: Color2::default(),
            selected_colors: !Color2::default(),
            unselected_colors: Color2::default(),
            cursor_colors: Color2::default(),
            box_colors: !Color2::default(),
            bg: Color::Black,
            title_y: 1,
            pad_after_title: 2,
            pad_after_box: 2,
            pad_after_ok: 1,
        }
    }

    /// Gets user input without possibility of canceling it.
    pub async fn select(&mut self) -> Result<GString> {
        let mut buffer = self.buffer.chars().collect::<Vec<_>>();
        let mut cursor = 0;

        self.render(&buffer, cursor, InputDialogItem::Ok, false).await?;
        let mut joined = String::new();

        loop {
            match self.term.listen_event().await? {
                Event::Key(KeyEvent {
                    main_key: Key::Left,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if cursor > 0 {
                        cursor -= 1;
                        self.render_input_box(
                            &mut self.term.lock_screen().await,
                            &buffer,
                            cursor,
                        )?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Right,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if cursor < buffer.len() {
                        cursor += 1;
                        self.render_input_box(
                            &mut self.term.lock_screen().await,
                            &buffer,
                            cursor,
                        )?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => break,

                Event::Key(KeyEvent {
                    main_key: Key::Backspace,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if cursor > 0 {
                        cursor -= 1;
                        buffer.remove(cursor);
                        self.render_input_box(
                            &mut self.term.lock_screen().await,
                            &buffer,
                            cursor,
                        )?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Char(ch),
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if (self.filter)(ch) {
                        joined.clear();
                        joined.push('a');
                        joined.push(ch);
                        if joined.graphemes(true).count() > 1 {
                            joined.clear();
                            joined.extend(buffer.iter());
                            joined.push(ch);
                            joined.clear();
                            joined.extend(buffer.iter());
                            joined.push(ch);
                            let length = joined.graphemes(true).count() as Nat;
                            if length <= self.max {
                                buffer.insert(cursor, ch);
                                cursor += 1;
                                self.render_input_box(
                                    &mut self.term.lock_screen().await,
                                    &buffer,
                                    cursor,
                                )?;
                            }
                        }
                    }
                },

                Event::Resize(_) => {
                    self.render(&buffer, cursor, InputDialogItem::Ok, false)
                        .await?;
                },

                _ => (),
            }
        }

        let string = buffer.into_iter().collect::<String>();
        Ok(gstring![&string])
    }

    /// Gets user input with the user possibly canceling it.
    pub async fn select_with_cancel(&mut self) -> Result<Option<GString>> {
        let mut selected = InputDialogItem::Ok;
        let mut buffer = self.buffer.chars().collect::<Vec<_>>();
        let mut cursor = 0;
        self.render(&buffer, cursor, selected, true).await?;
        let mut joined = String::new();

        loop {
            match self.term.listen_event().await? {
                Event::Key(KeyEvent {
                    main_key: Key::Esc,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    selected = InputDialogItem::Cancel;
                    break;
                },

                Event::Key(KeyEvent {
                    main_key: Key::Left,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if cursor > 0 {
                        cursor -= 1;
                        self.render_input_box(
                            &mut self.term.lock_screen().await,
                            &buffer,
                            cursor,
                        )?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Right,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if cursor < buffer.len() {
                        cursor += 1;
                        self.render_input_box(
                            &mut self.term.lock_screen().await,
                            &buffer,
                            cursor,
                        )?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Up,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    selected = InputDialogItem::Ok;
                    self.render_item(
                        &mut self.term.lock_screen().await,
                        InputDialogItem::Ok,
                        selected,
                    )?;
                    self.render_item(
                        &mut self.term.lock_screen().await,
                        InputDialogItem::Cancel,
                        selected,
                    )?;
                },

                Event::Key(KeyEvent {
                    main_key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    selected = InputDialogItem::Cancel;
                    self.render_item(
                        &mut self.term.lock_screen().await,
                        InputDialogItem::Ok,
                        selected,
                    )?;
                    self.render_item(
                        &mut self.term.lock_screen().await,
                        InputDialogItem::Cancel,
                        selected,
                    )?;
                },

                Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => break,

                Event::Key(KeyEvent {
                    main_key: Key::Backspace,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if cursor > 0 {
                        cursor -= 1;
                        buffer.remove(cursor);
                        self.render_input_box(
                            &mut self.term.lock_screen().await,
                            &buffer,
                            cursor,
                        )?;
                    }
                },

                Event::Key(KeyEvent {
                    main_key: Key::Char(ch),
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if (self.filter)(ch) {
                        joined.clear();
                        joined.push('a');
                        joined.push(ch);
                        if joined.graphemes(true).count() > 1 {
                            joined.clear();
                            joined.extend(buffer.iter());
                            joined.push(ch);
                            let length = joined.graphemes(true).count() as Nat;
                            if length <= self.max {
                                buffer.insert(cursor, ch);
                                cursor += 1;
                                self.render_input_box(
                                    &mut self.term.lock_screen().await,
                                    &buffer,
                                    cursor,
                                )?;
                            }
                        }
                    }
                },

                Event::Resize(_) => {
                    self.render(&buffer, cursor, selected, true).await?;
                },

                _ => (),
            }
        }

        Ok(match selected {
            InputDialogItem::Ok => {
                let string = buffer.into_iter().collect::<String>();
                Some(gstring![&string])
            },
            InputDialogItem::Cancel => None,
        })
    }

    async fn render(
        &self,
        buffer: &[char],
        cursor: usize,
        selected: InputDialogItem,
        has_cancel: bool,
    ) -> Result<()> {
        let mut screen = self.term.lock_screen().await;
        screen.clear(self.bg);
        let style = Style::new()
            .left_margin(1)
            .right_margin(1)
            .align(1, 2)
            .max_height(self.pad_after_title.saturating_add(1))
            .top_margin(self.title_y);
        screen.styled_text(&self.title, style)?;
        self.render_input_box(&mut screen, buffer, cursor)?;
        self.render_item(&mut screen, InputDialogItem::Ok, selected)?;
        if has_cancel {
            self.render_item(&mut screen, InputDialogItem::Cancel, selected)?;
        }

        Ok(())
    }

    fn render_input_box(
        &self,
        screen: &mut terminal::Screen,
        buffer: &[char],
        cursor: usize,
    ) -> Result<()> {
        let mut field = buffer.iter().collect::<String>();
        let additional = self.max as usize - buffer.len();
        field.reserve(additional);

        for _ in 0 .. additional {
            field.push_str(" ");
        }

        let style = Style::new()
            .align(1, 2)
            .top_margin(self.y_of_box())
            .colors(self.box_colors);
        let string = gstring![&field];
        screen.styled_text(&string, style)?;


        let width = screen.handle().screen_size().x;
        let correction = (self.max % 2 + width % 2 + 1) as usize;
        let length = field.graphemes(true).count() - correction % 2;

        field.clear();

        for i in 0 .. length + 1 {
            if i == cursor {
                field.push('¯')
            } else {
                field.push(' ')
            }
        }

        let style = Style::new()
            .align(1, 2)
            .top_margin(self.y_of_box() + 1)
            .left_margin(1)
            .colors(self.cursor_colors);
        let string = gstring![&field];
        screen.styled_text(&string, style)?;

        Ok(())
    }

    fn render_item(
        &self,
        screen: &mut terminal::Screen,
        item: InputDialogItem,
        selected: InputDialogItem,
    ) -> Result<()> {
        let (option, y) = match item {
            InputDialogItem::Ok => ("> OK <", self.y_of_ok()),
            InputDialogItem::Cancel => ("> CANCEL <", self.y_of_cancel()),
        };
        let colors = if item == selected {
            self.selected_colors
        } else {
            self.unselected_colors
        };

        let style = Style::new().align(1, 2).colors(colors).top_margin(y);
        let string = gstring![option];
        screen.styled_text(&string, style)?;

        Ok(())
    }

    fn y_of_box(&self) -> Nat {
        self.title_y + 1 + self.pad_after_title
    }

    fn y_of_ok(&self) -> Nat {
        self.y_of_box() + 2 + self.pad_after_box
    }

    fn y_of_cancel(&self) -> Nat {
        self.y_of_ok() + 1 + self.pad_after_ok
    }
}
