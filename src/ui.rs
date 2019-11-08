use crate::{
    backend::Backend,
    error::GameResult,
    iter_ext::IterExt,
    key::Key,
    orient::{Coord, Coord2D},
    render::{Color, TextSettings},
    term::{self, Terminal},
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
    Quit,
}

impl MenuItem for MainMenuItem {
    fn name(&self) -> &str {
        match self {
            MainMenuItem::NewGame => "NEW GAME",
            MainMenuItem::LoadGame => "LOAD GAME",

            MainMenuItem::Quit => {
                "QUIT**********************************************************\
                 **********************"
            },
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
        [MainMenuItem::NewGame, MainMenuItem::LoadGame, MainMenuItem::Quit]
            .into_iter()
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

    /// Asks for the user for an option, without cancel option.
    fn select<B>(
        &'menu self,
        term: &mut Terminal<B>,
    ) -> GameResult<&'menu Self::Item>
    where
        B: Backend,
    {
        let mut selected = 0;
        let mut start = 0;

        render(self, term, start, Some(selected), false)?;

        term.call(move |term| {
            let screen_end = screen_end(start, term.screen_size(), false);

            match term.key()? {
                Some(Key::Up) => {
                    if selected > 0 {
                        selected -= 1;
                        if selected < start {
                            start -= 1;
                        }
                        render(self, term, start, Some(selected), false)?;
                    }
                },

                Some(Key::Down) => {
                    if selected + 1 < self.items().count() as Coord {
                        selected += 1;
                        if selected >= screen_end {
                            start += 1;
                        }
                        render(self, term, start, Some(selected), false)?;
                    }
                },

                Some(Key::Enter) => {
                    return Ok(term::Stop(
                        self.items().nth(selected as usize).unwrap(),
                    ))
                },

                _ => {
                    if term.has_resized() {
                        render(self, term, start, Some(selected), false)?;
                    }
                },
            }

            Ok(term::Continue)
        })
    }

    /// Asks for the user for an option, with cancel option.
    fn select_with_cancel<B>(
        &'menu self,
        term: &mut Terminal<B>,
    ) -> GameResult<Option<&'menu Self::Item>>
    where
        B: Backend,
    {
        let mut selected = 0;
        let empty = self.items().next().is_none();
        let mut is_cancel = empty;
        let mut start = 0;

        render(self, term, start, Some(selected).filter(|_| !is_cancel), true)?;

        term.call(move |term| {
            let screen_end = screen_end(start, term.screen_size(), false);

            match term.key()? {
                Some(Key::Up) => {
                    if is_cancel && !empty {
                        is_cancel = false;
                        render(self, term, start, Some(selected), true)?;
                    } else if selected > 0 {
                        selected -= 1;
                        if selected < start {
                            start -= 1;
                        }
                        render(
                            self,
                            term,
                            start,
                            Some(selected).filter(|_| !is_cancel),
                            true,
                        )?;
                    }
                },

                Some(Key::Down) => {
                    if selected + 1 < self.items().count() as Coord {
                        selected += 1;
                        if selected >= screen_end {
                            start += 1;
                        }
                        render(
                            self,
                            term,
                            start,
                            Some(selected).filter(|_| !is_cancel),
                            true,
                        )?;
                    } else if !is_cancel {
                        is_cancel = true;
                        render(self, term, start, None, true)?;
                    }
                },

                Some(Key::Left) => {
                    if !is_cancel {
                        is_cancel = true;
                        render(self, term, start, None, true)?;
                    }
                },

                Some(Key::Right) => {
                    if is_cancel && !empty {
                        is_cancel = false;
                        render(self, term, start, Some(selected), true)?;
                    }
                },

                Some(Key::Enter) => {
                    return Ok(term::Stop(if is_cancel {
                        None
                    } else {
                        Some(self.items().nth(selected as usize).unwrap())
                    }))
                },

                _ => {
                    if term.has_resized() {
                        render(
                            self,
                            term,
                            start,
                            Some(selected).filter(|_| !is_cancel),
                            true,
                        )?;
                    }
                },
            }

            Ok(term::Continue)
        })
    }
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

fn render<'menu, M, B>(
    menu: &'menu M,
    term: &mut Terminal<B>,
    start: Coord,
    selected: Option<Coord>,
    cancel: bool,
) -> GameResult<()>
where
    B: Backend,
    M: Menu<'menu> + ?Sized,
{
    let screen_size = term.screen_size();

    term.clear_screen()?;
    term.text(menu.title(), 1, TextSettings::new().align(1, 2))?;
    let range = range_of_screen(start, term.screen_size(), cancel);
    for (i, option) in menu.items().enumerate().slice(range) {
        let i = i as Coord;
        let is_selected = Some(i) == selected;
        render_option(term, option, y_of_option(start, i), is_selected)?;
    }

    if cancel {
        render_cancel(term, screen_size.y, selected.is_none())?;
    }

    Ok(())
}

fn render_option<B, M>(
    term: &mut Terminal<B>,
    option: &M,
    y: Coord,
    selected: bool,
) -> GameResult<()>
where
    B: Backend,
    M: MenuItem,
{
    if selected {
        term.setbg(Color::White)?;
        term.setfg(Color::Black)?;
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
    term.text(&formatted, y, TextSettings::new().align(1, 2))?;

    term.setbg(Color::Black)?;
    term.setfg(Color::White)?;

    Ok(())
}

fn render_cancel<B>(
    term: &mut Terminal<B>,
    cancel_y: Coord,
    selected: bool,
) -> GameResult<()>
where
    B: Backend,
{
    if selected {
        term.setbg(Color::White)?;
        term.setfg(Color::Black)?;
    }

    term.text("> Cancel <", cancel_y - 2, TextSettings::new().align(1, 3))?;

    term.setbg(Color::Black)?;
    term.setfg(Color::White)?;

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
    pub fn run<B>(&self, term: &mut Terminal<B>) -> GameResult<()>
    where
        B: Backend,
    {
        self.render(term)?;
        term.call(move |term| {
            if term.has_resized() {
                self.render(term)?;
            }

            Ok(match term.key()? {
                Some(Key::Enter) => term::Stop(()),
                _ => term::Continue,
            })
        })
    }

    fn render<B>(&self, term: &mut Terminal<B>) -> GameResult<()>
    where
        B: Backend,
    {
        term.clear_screen()?;
        term.text(self.title, 1, TextSettings::new().align(1, 2))?;
        let pos = term.text(self.message, 3, self.settings)?;

        term.setbg(Color::White)?;
        term.setfg(Color::Black)?;

        term.text("> OK <", pos + 2, TextSettings::new().align(1, 2))?;

        term.setbg(Color::Black)?;
        term.setfg(Color::White)?;

        Ok(())
    }
}
