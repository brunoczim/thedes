use crate::{
    backend::{check_screen_size, Backend},
    error::GameResult,
    iter_ext::IterExt,
    key::Key,
    orient::{Coord, Coord2D},
    render::Color,
    timer,
};
use std::{ops::Range, slice, time::Duration};

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
            MainMenuItem::Quit => "QUIT",
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
    fn select<B>(&'menu self, backend: &mut B) -> GameResult<&'menu Self::Item>
    where
        B: Backend,
    {
        backend.clear_screen()?;
        let mut term_size = backend.term_size()?;
        let mut selected = 0;
        let mut start = 0;

        render(self, backend, start, Some(selected), term_size, false)?;

        timer::tick(Duration::from_millis(50), move || {
            check_screen_size(backend, &mut term_size)?;

            match backend.try_get_key()? {
                Some(Key::Up) => {
                    if selected > 0 {
                        selected -= 1;
                        if selected < start {
                            start -= 1;
                        }
                        render(
                            self,
                            backend,
                            start,
                            Some(selected),
                            term_size,
                            false,
                        )?;
                    }
                },

                Some(Key::Down) => {
                    if selected + 1 < self.items().count() as Coord {
                        selected += 1;
                        if selected >= screen_end(start, term_size, false) {
                            start += 1;
                        }
                        render(
                            self,
                            backend,
                            start,
                            Some(selected),
                            term_size,
                            false,
                        )?;
                    }
                },

                Some(Key::Enter) => {
                    return Ok(timer::Stop(
                        self.items().nth(selected as usize).unwrap(),
                    ))
                },

                _ => (),
            }

            Ok(timer::Continue)
        })
    }

    fn select_with_cancel<B>(
        &'menu self,
        backend: &mut B,
    ) -> GameResult<Option<&'menu Self::Item>>
    where
        B: Backend,
    {
        backend.clear_screen()?;
        let mut term_size = backend.term_size()?;
        let mut selected = 0;
        let mut is_cancel = false;
        let mut start = 0;

        render(
            self,
            backend,
            start,
            Some(selected).filter(|_| is_cancel),
            term_size,
            true,
        )?;

        timer::tick(Duration::from_millis(50), move || {
            check_screen_size(backend, &mut term_size)?;

            match backend.try_get_key()? {
                Some(Key::Up) => {
                    if is_cancel {
                        is_cancel = false;
                        render(
                            self,
                            backend,
                            start,
                            Some(selected),
                            term_size,
                            true,
                        )?;
                    } else if selected > 0 {
                        selected -= 1;
                        if selected < start {
                            start -= 1;
                        }
                        render(
                            self,
                            backend,
                            start,
                            Some(selected).filter(|_| is_cancel),
                            term_size,
                            true,
                        )?;
                    }
                },

                Some(Key::Down) => {
                    if selected + 1 < self.items().count() as Coord {
                        selected += 1;
                        if selected >= screen_end(start, term_size, false) {
                            start += 1;
                        }
                        render(
                            self,
                            backend,
                            start,
                            Some(selected).filter(|_| is_cancel),
                            term_size,
                            true,
                        )?;
                    } else if !is_cancel {
                        is_cancel = true;
                        render(self, backend, start, None, term_size, true)?;
                    }
                },

                Some(Key::Left) => {
                    if !is_cancel {
                        is_cancel = true;
                        render(self, backend, start, None, term_size, true)?;
                    }
                },

                Some(Key::Right) => {
                    if is_cancel {
                        is_cancel = false;
                        render(
                            self,
                            backend,
                            start,
                            Some(selected),
                            term_size,
                            true,
                        )?;
                    }
                },

                Some(Key::Enter) => {
                    return Ok(timer::Stop(if is_cancel {
                        None
                    } else {
                        Some(self.items().nth(selected as usize).unwrap())
                    }))
                },

                _ => (),
            }

            Ok(timer::Continue)
        })
    }
}

fn y_of_option(start: Coord, option: Coord) -> Coord {
    (option - start) * OPTION_HEIGHT + TITLE_HEIGHT + OPTION_HEIGHT / 2
}

fn screen_end(start: Coord, term_size: Coord2D, cancel: bool) -> Coord {
    let cancel = if cancel { 1 } else { 0 };
    start + (term_size.y - TITLE_HEIGHT - 4 * cancel) / OPTION_HEIGHT
}

fn range_of_screen(
    start: Coord,
    term_size: Coord2D,
    cancel: bool,
) -> Range<usize> {
    start as usize .. screen_end(start, term_size, cancel) as usize
}

fn render<'menu, M, B>(
    menu: &'menu M,
    backend: &mut B,
    start: Coord,
    selected: Option<Coord>,
    term_size: Coord2D,
    cancel: bool,
) -> GameResult<()>
where
    B: Backend,
    M: Menu<'menu> + ?Sized,
{
    backend.aligned_text(menu.title(), 1, 2, 1)?;
    let range = range_of_screen(start, term_size, cancel);
    for (i, option) in menu.items().enumerate().slice(range) {
        let i = i as Coord;
        let is_selected = Some(i) == selected;
        render_option(backend, option, y_of_option(start, i), is_selected)?;
    }

    if cancel {
        render_cancel(backend, term_size.y, selected.is_none())?;
    }

    Ok(())
}

fn render_option<B, M>(
    backend: &mut B,
    option: &M,
    y: Coord,
    selected: bool,
) -> GameResult<()>
where
    B: Backend,
    M: MenuItem,
{
    if selected {
        backend.setbg(Color::White)?;
        backend.setfg(Color::Black)?;
    }

    let formatted = format!("> {} <", option.name());
    backend.aligned_text(&formatted, 1, 2, y)?;

    backend.setbg(Color::Black)?;
    backend.setfg(Color::White)?;

    Ok(())
}

fn render_cancel<B>(
    backend: &mut B,
    cancel_y: Coord,
    selected: bool,
) -> GameResult<()>
where
    B: Backend,
{
    if selected {
        backend.setbg(Color::White)?;
        backend.setfg(Color::Black)?;
    }

    backend.aligned_text("> Cancel <", 1, 3, cancel_y)?;

    backend.setbg(Color::Black)?;
    backend.setfg(Color::White)?;

    Ok(())
}
