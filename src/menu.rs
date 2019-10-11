use crate::{
    backend::{check_screen_size, Backend},
    error::GameResult,
    key::Key,
    orient::{Coord, Coord2D},
    render::Color,
    timer,
};
use std::time::Duration;

/// The main menu of a game.
#[derive(Debug)]
pub enum MainMenu {
    NewGame,
    Quit,
}

impl MainMenu {
    /// A list of all menu items.
    pub const ITEMS: &'static [Self] = &[MainMenu::NewGame, MainMenu::Quit];
}

impl Menu for MainMenu {
    fn option_name(&self) -> &str {
        match self {
            MainMenu::NewGame => "NEW GAME",
            MainMenu::Quit => "QUIT",
        }
    }
}

/// A type that is an option of a menu.
pub trait Menu: Sized {
    /// Converts the option to a showable name.
    fn option_name(&self) -> &str;

    /// Selects an option from user input.
    fn select<'opts, B>(
        options: &'opts [Self],
        backend: &mut B,
    ) -> GameResult<&'opts Self>
    where
        B: Backend,
    {
        let mut term_size = backend.term_size()?;
        let mut selected = 0;
        render_menu(backend, options, selected, term_size)?;

        timer::tick(Duration::from_millis(50), move || {
            if check_screen_size(backend, &mut term_size)? {
                render_menu(backend, options, selected, term_size)?;
            }

            match backend.try_get_key()? {
                Some(Key::Up) => {
                    if selected > 0 {
                        update_rendered_menu(
                            backend,
                            options,
                            selected,
                            selected - 1,
                            term_size,
                        )?;
                        selected -= 1;
                    }
                },

                Some(Key::Down) => {
                    if selected + 1 < options.len() {
                        update_rendered_menu(
                            backend,
                            options,
                            selected,
                            selected + 1,
                            term_size,
                        )?;
                        selected += 1;
                    }
                },

                Some(Key::Enter) => return Ok(timer::Stop(&options[selected])),

                _ => (),
            }

            Ok(timer::Continue)
        })
    }
}

fn update_rendered_menu<M, B>(
    backend: &mut B,
    options: &[M],
    prev_selected: usize,
    selected: usize,
    term_size: Coord2D,
) -> GameResult<()>
where
    B: Backend,
    M: Menu,
{
    render_option(
        backend,
        prev_selected,
        &options[prev_selected],
        false,
        term_size,
    )?;
    render_option(backend, selected, &options[selected], true, term_size)?;
    Ok(())
}

fn render_option<B, M>(
    backend: &mut B,
    index: usize,
    option: &M,
    selected: bool,
    term_size: Coord2D,
) -> GameResult<()>
where
    M: Menu,
    B: Backend,
{
    let pos = (term_size.x - option.option_name().len() as Coord) / 2 - 2;
    backend.goto(Coord2D { x: pos, y: (index * 2 + 1) as Coord })?;
    if selected {
        backend.setfg(Color::Black)?;
        backend.setbg(Color::White)?;
        write!(backend, "> {} <", option.option_name())?;
        backend.setfg(Color::White)?;
        backend.setbg(Color::Black)?;
    } else {
        write!(backend, "  {}  ", option.option_name())?;
    }
    Ok(())
}

fn render_menu<M, B>(
    backend: &mut B,
    options: &[M],
    selected: usize,
    term_size: Coord2D,
) -> GameResult<()>
where
    M: Menu,
    B: Backend,
{
    backend.clear_screen()?;

    for (i, option) in options.iter().enumerate() {
        render_option(backend, i, option, i == selected, term_size)?;
    }

    Ok(())
}
