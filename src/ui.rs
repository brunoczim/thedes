use crate::{
    error::GameResult,
    input::{Event, Key, KeyEvent},
    iter_ext::IterExt,
    orient::{Coord, Coord2D},
    render::{Color, TextSettings, MIN_SCREEN},
    terminal,
};
use std::{ops::Range, slice};
use unicode_segmentation::UnicodeSegmentation;

const TITLE_HEIGHT: Coord = 3;
const OPTION_HEIGHT: Coord = 2;

/// The item of a game's main menu.
#[derive(Debug, Copy, Clone)]
pub enum MainMenuItem {
    NewGame,
    LoadGame,
    DeleteGame,
    Exit,
}

impl MenuItem for MainMenuItem {
    fn name(&self) -> &str {
        match self {
            MainMenuItem::NewGame => "NEW GAME",
            MainMenuItem::LoadGame => "LOAD GAME",
            MainMenuItem::DeleteGame => "DELETE GAME",
            MainMenuItem::Exit => "EXIT",
        }
    }
}

/// The main menu of a game.
#[derive(Debug, Copy, Clone)]
pub struct MainMenu;

impl<'menu> Menu<'menu> for MainMenu {
    type Item = MainMenuItem;
    type Iter = slice::Iter<'menu, Self::Item>;

    fn items(&'menu self) -> Self::Iter {
        [
            MainMenuItem::NewGame,
            MainMenuItem::LoadGame,
            MainMenuItem::DeleteGame,
            MainMenuItem::Exit,
        ]
        .iter()
    }

    fn title(&'menu self) -> &'menu str {
        "T H E D E S"
    }
}

/// A type that is an option of a menu.
pub trait MenuItem {
    /// Converts the option to a showable name.
    fn name(&self) -> &str;
}

/// A showable menu.
pub trait Menu<'menu>
where
    Self: 'menu,
{
    /// An option of this menu (without cancel on it).
    type Item: MenuItem + 'menu;
    /// An iterator over all options.
    type Iter: Iterator<Item = &'menu Self::Item>;

    /// Title of this menu.
    fn title(&'menu self) -> &'menu str;
    /// A list of all menu items.
    fn items(&'menu self) -> Self::Iter;
}

/// Asks for the user for an option, without cancel option.
pub async fn menu_select<'menu, M>(
    menu: &'menu M,
    term: &mut terminal::Handle,
) -> GameResult<&'menu <M as Menu<'menu>>::Item>
where
    M: Menu<'menu>,
{
    let mut selected = 0;
    let mut start = 0;

    render_menu(menu, term, start, Some(selected), false).await?;
    let mut last_row = screen_end(start, term.screen_size(), false);
    let count = menu.items().count() as Coord;

    let ret = loop {
        match term.listen_event().await {
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
                    }
                    render_menu(menu, term, start, Some(selected), false)
                        .await?;
                    term.flush().await?;
                }
            },

            Event::Key(KeyEvent {
                main_key: Key::Down,
                alt: false,
                ctrl: false,
                shift: false,
            }) => {
                if selected + 1 < count {
                    selected += 1;
                    if selected >= last_row {
                        start += 1;
                    }
                    render_menu(menu, term, start, Some(selected), false)
                        .await?;
                    term.flush().await?;
                }
            },

            Event::Key(KeyEvent {
                main_key: Key::Enter,
                alt: false,
                ctrl: false,
                shift: false,
            }) => {
                break menu
                    .items()
                    .nth(selected as usize)
                    .expect("Inconsistent menu");
            },

            Event::Resize(evt) => {
                render_menu(menu, term, start, Some(selected), false).await?;
                last_row = screen_end(start, evt.size, false);
            },

            _ => (),
        }
    };

    Ok(ret)
}

/// Asks for the user for an option, with cancel option.
pub async fn menu_select_with_cancel<'menu, M>(
    menu: &'menu M,
    term: &mut terminal::Handle,
) -> GameResult<Option<&'menu <M as Menu<'menu>>::Item>>
where
    M: Menu<'menu>,
{
    let mut selected = 0;
    let empty = menu.items().next().is_none();
    let mut is_cancel = empty;
    let mut start = 0;

    render_menu(menu, term, start, Some(selected).filter(|_| !is_cancel), true)
        .await?;
    let mut last_row = screen_end(start, term.screen_size(), false);
    let count = menu.items().count() as Coord;

    let ret = loop {
        match term.listen_event().await {
            Event::Key(KeyEvent {
                main_key: Key::Up,
                ctrl: false,
                alt: false,
                shift: false,
            }) => {
                if is_cancel && !empty {
                    is_cancel = false;
                    render_menu(menu, term, start, Some(selected), true)
                        .await?;
                } else if selected > 0 {
                    selected -= 1;
                    if selected < start {
                        start -= 1;
                    }
                    render_menu(
                        menu,
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
                if selected + 1 < count {
                    selected += 1;
                    if selected >= last_row {
                        start += 1;
                    }
                    render_menu(
                        menu,
                        term,
                        start,
                        Some(selected).filter(|_| !is_cancel),
                        true,
                    )
                    .await?;
                } else if !is_cancel {
                    is_cancel = true;
                    render_menu(menu, term, start, None, true).await?;
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
                    render_menu(menu, term, start, None, true).await?;
                }
            },

            Event::Key(KeyEvent {
                main_key: Key::Right,
                ctrl: false,
                alt: false,
                shift: false,
            }) => {
                if is_cancel && !empty {
                    is_cancel = false;
                    render_menu(menu, term, start, Some(selected), true)
                        .await?;
                }
            },

            Event::Key(KeyEvent {
                main_key: Key::Enter,
                ctrl: false,
                alt: false,
                shift: false,
            }) => {
                break if is_cancel {
                    None
                } else {
                    Some(menu.items().nth(selected as usize).unwrap())
                }
            },

            Event::Resize(evt) => {
                render_menu(menu, term, start, Some(selected), false).await?;
                last_row = screen_end(start, evt.size, false);
            },

            _ => (),
        }
    };

    Ok(ret)
}

fn y_of_option(start: Coord, option: Coord) -> Coord {
    (option - start) * OPTION_HEIGHT + TITLE_HEIGHT + OPTION_HEIGHT / 2
}

fn screen_end(start: Coord, screen_size: Coord2D, cancel: bool) -> Coord {
    let cancel = if cancel { 1 } else { 0 };
    start + (screen_size.y - TITLE_HEIGHT - 4 * cancel) / OPTION_HEIGHT
}

fn range_of_screen(
    start: Coord,
    screen_size: Coord2D,
    cancel: bool,
) -> Range<usize> {
    start as usize .. screen_end(start, screen_size, cancel) as usize
}

async fn render_menu<'menu, M>(
    menu: &'menu M,
    term: &mut terminal::Handle,
    start: Coord,
    selected: Option<Coord>,
    cancel: bool,
) -> GameResult<()>
where
    M: Menu<'menu> + ?Sized,
{
    let screen_size = term.screen_size();

    term.clear_screen()?;
    term.aligned_text(menu.title(), 1, TextSettings::new().align(1, 2))?;
    let range = range_of_screen(start, term.screen_size(), cancel);
    for (i, option) in menu.items().enumerate().slice(range) {
        let i = i as Coord;
        let is_selected = Some(i) == selected;
        render_option(term, option, y_of_option(start, i), is_selected)?;
    }

    if cancel {
        render_cancel(term, screen_size.y, selected.is_none())?;
    }

    term.flush().await?;
    Ok(())
}

fn render_option<M>(
    term: &mut terminal::Handle,
    option: &M,
    y: Coord,
    selected: bool,
) -> GameResult<()>
where
    M: MenuItem,
{
    if selected {
        term.set_bg(Color::White)?;
        term.set_fg(Color::Black)?;
    }

    let mut buf = String::from(option.name());
    let indices =
        buf.grapheme_indices(true).map(|(i, _)| i).collect::<Vec<_>>();
    let screen = term.screen_size();

    if indices.len() as Coord % 2 != screen.x % 2 {
        buf.push_str(" ");
    }

    if screen.x - 4 < indices.len() as Coord {
        buf.truncate(indices[screen.x as usize - 7]);
        buf.push_str("...");
    }

    let formatted = format!("> {} <", buf);
    term.aligned_text(&formatted, y, TextSettings::new().align(1, 2))?;

    term.set_bg(Color::Black)?;
    term.set_fg(Color::White)?;

    Ok(())
}

fn render_cancel(
    term: &mut terminal::Handle,
    cancel_y: Coord,
    selected: bool,
) -> GameResult<()> {
    if selected {
        term.set_bg(Color::White)?;
        term.set_fg(Color::Black)?;
    }

    term.aligned_text(
        "> Cancel <",
        cancel_y - 2,
        TextSettings::new().align(1, 3),
    )?;

    term.set_bg(Color::Black)?;
    term.set_fg(Color::White)?;

    Ok(())
}

/// An info dialog, with just an Ok option.
#[derive(Debug, Copy, Clone)]
pub struct InfoDialog<'msg> {
    /// Title to be shown.
    pub title: &'msg str,
    /// Long text message to be shown.
    pub message: &'msg str,
    /// Settings such as margin and alignment.
    pub settings: TextSettings,
}

impl<'msg> InfoDialog<'msg> {
    /// Runs this dialog showing it to the user, awaiting OK!
    pub async fn run(&self, term: &mut terminal::Handle) -> GameResult<()> {
        self.render(term).await?;

        loop {
            match term.listen_event().await {
                Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => break Ok(()),

                Event::Resize(_) => self.render(term).await?,

                _ => (),
            }
        }
    }

    async fn render(&self, term: &mut terminal::Handle) -> GameResult<()> {
        term.clear_screen()?;
        term.aligned_text(self.title, 1, TextSettings::new().align(1, 2))?;
        let pos = term.aligned_text(self.message, 3, self.settings)?;

        term.set_bg(Color::White)?;
        term.set_fg(Color::Black)?;

        term.aligned_text("> OK <", pos + 2, TextSettings::new().align(1, 2))?;

        term.set_bg(Color::Black)?;
        term.set_fg(Color::White)?;

        term.flush().await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputDialogItem {
    Ok,
    Cancel,
}

/// A dialog asking for user input, possibly filtered.
pub struct InputDialog<'buf, F>
where
    F: FnMut(char) -> bool,
{
    title: &'buf str,
    buffer: &'buf str,
    max: Coord,
    filter: F,
}

impl<'buf, F> InputDialog<'buf, F>
where
    F: FnMut(char) -> bool,
{
    /// Creates a new input dialog, with the given title, initial buffer,
    /// maximum input size, and filter function.
    pub fn new(
        title: &'buf str,
        buffer: &'buf str,
        max: Coord,
        filter: F,
    ) -> Self {
        Self { title, buffer, filter, max: max.min(MIN_SCREEN.x - 1) }
    }

    /// Gets user input without possibility of canceling it.
    pub async fn select(
        &mut self,
        term: &mut terminal::Handle,
    ) -> GameResult<String> {
        let mut buffer = self.buffer.chars().collect::<Vec<_>>();
        let mut cursor = 0;
        let mut title_y = self
            .render(term, &buffer, cursor, InputDialogItem::Ok, false)
            .await?;
        let mut joined = String::new();

        loop {
            match term.listen_event().await {
                Event::Key(KeyEvent {
                    main_key: Key::Left,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if cursor > 0 {
                        cursor -= 1;
                        self.render_input_box(term, title_y, &buffer, cursor)?;
                        term.flush().await?;
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
                        self.render_input_box(term, title_y, &buffer, cursor)?;
                        term.flush().await?;
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
                        self.render_input_box(term, title_y, &buffer, cursor)?;
                        term.flush().await?;
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
                        joined.extend(buffer.iter());
                        joined.push(ch);
                        let length = joined.graphemes(true).count() as Coord;
                        if length <= self.max {
                            buffer.insert(cursor, ch);
                            cursor += 1;
                            self.render_input_box(
                                term, title_y, &buffer, cursor,
                            )?;
                            term.flush().await?;
                        }
                    }
                },

                Event::Resize(_) => {
                    title_y = self
                        .render(
                            term,
                            &buffer,
                            cursor,
                            InputDialogItem::Ok,
                            false,
                        )
                        .await?;
                },

                _ => (),
            }
        }

        Ok(buffer.into_iter().collect())
    }

    /// Gets user input with the user possibly canceling it.
    pub async fn select_with_cancel(
        &mut self,
        term: &mut terminal::Handle,
    ) -> GameResult<Option<String>> {
        let mut selected = InputDialogItem::Ok;
        let mut buffer = self.buffer.chars().collect::<Vec<_>>();
        let mut cursor = 0;
        let mut title_y =
            self.render(term, &buffer, cursor, selected, true).await?;
        let mut joined = String::new();

        loop {
            match term.listen_event().await {
                Event::Key(KeyEvent {
                    main_key: Key::Left,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    if cursor > 0 {
                        cursor -= 1;
                        self.render_input_box(term, title_y, &buffer, cursor)?;
                        term.flush().await?;
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
                        self.render_input_box(term, title_y, &buffer, cursor)?;
                        term.flush().await?;
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
                        term,
                        InputDialogItem::Ok,
                        selected,
                        title_y,
                    )?;
                    self.render_item(
                        term,
                        InputDialogItem::Cancel,
                        selected,
                        title_y,
                    )?;
                    term.flush().await?;
                },

                Event::Key(KeyEvent {
                    main_key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }) => {
                    selected = InputDialogItem::Cancel;
                    self.render_item(
                        term,
                        InputDialogItem::Ok,
                        selected,
                        title_y,
                    )?;
                    self.render_item(
                        term,
                        InputDialogItem::Cancel,
                        selected,
                        title_y,
                    )?;
                    term.flush().await?;
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
                        self.render_input_box(term, title_y, &buffer, cursor)?;
                        term.flush().await?;
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
                        joined.extend(buffer.iter());
                        joined.push(ch);
                        let length = joined.graphemes(true).count() as Coord;
                        if length <= self.max {
                            buffer.insert(cursor, ch);
                            cursor += 1;
                            self.render_input_box(
                                term, title_y, &buffer, cursor,
                            )?;
                            term.flush().await?;
                        }
                    }
                },

                Event::Resize(_) => {
                    title_y = self
                        .render(term, &buffer, cursor, selected, true)
                        .await?;
                },

                _ => (),
            }
        }

        Ok(match selected {
            InputDialogItem::Ok => Some(buffer.into_iter().collect()),
            InputDialogItem::Cancel => None,
        })
    }

    async fn render(
        &self,
        term: &mut terminal::Handle,
        buffer: &[char],
        cursor: usize,
        selected: InputDialogItem,
        has_cancel: bool,
    ) -> GameResult<Coord> {
        term.clear_screen()?;
        let title_y = term.aligned_text(
            self.title,
            1,
            TextSettings { lmargin: 1, rmargin: 1, num: 1, den: 2 },
        )?;
        self.render_input_box(term, title_y, buffer, cursor)?;
        self.render_item(term, InputDialogItem::Ok, selected, title_y)?;
        if has_cancel {
            self.render_item(term, InputDialogItem::Cancel, selected, title_y)?;
        }

        term.flush().await?;

        Ok(title_y)
    }

    fn render_input_box(
        &self,
        term: &mut terminal::Handle,
        title_y: Coord,
        buffer: &[char],
        cursor: usize,
    ) -> GameResult<()> {
        term.set_fg(Color::Black)?;
        term.set_bg(Color::LightGrey)?;
        let mut field = buffer.iter().collect::<String>();
        let additional = self.max as usize - buffer.len();
        field.reserve(additional);

        for _ in 0 .. additional {
            field.push_str(" ");
        }

        term.aligned_text(
            &field,
            Self::y_of_input(title_y),
            TextSettings::new().align(1, 2),
        )?;

        let length = field.graphemes(true).count();

        field.clear();
        term.set_fg(Color::White)?;
        term.set_bg(Color::Black)?;

        for i in 0 .. length + 1 {
            if i == cursor {
                field.push('¯')
            } else {
                field.push(' ')
            }
        }

        term.aligned_text(
            &field,
            Self::y_of_input(title_y) + 1,
            TextSettings::new().align(1, 2).lmargin(1),
        )?;

        Ok(())
    }

    fn render_item(
        &self,
        term: &mut terminal::Handle,
        item: InputDialogItem,
        selected: InputDialogItem,
        title_y: Coord,
    ) -> GameResult<()> {
        if selected == item {
            term.set_fg(Color::Black)?;
            term.set_bg(Color::White)?;
        }

        let (option, y) = match item {
            InputDialogItem::Ok => ("> OK <", title_y + 6),
            InputDialogItem::Cancel => ("> CANCEL <", title_y + 8),
        };

        term.aligned_text(&option, y, TextSettings::new().align(1, 2))?;

        term.set_fg(Color::White)?;
        term.set_bg(Color::Black)?;

        Ok(())
    }

    fn y_of_input(title_y: Coord) -> Coord {
        title_y + 2
    }
}
