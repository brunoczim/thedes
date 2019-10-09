use crate::{
    backend::{check_screen_size, Backend},
    key::Key,
    orient::{Coord, Coord2D},
    render::Color,
};
use std::io;

/// A type that is an option of a menu.
pub trait Menu: Sized {
    /// Converts the option to a showable name.
    fn option_name(&self) -> &str;

    /// Selects an option from user input.
    fn select<'opts, B>(
        options: &'opts [Self],
        backend: &mut B,
    ) -> io::Result<&'opts Self>
    where
        B: Backend,
    {
        let mut term_size = backend.term_size()?;
        let mut selected = 0;
        render_menu(backend, options, selected, term_size)?;

        loop {
            if check_screen_size(backend, &mut term_size)? {
                render_menu(backend, options, selected, term_size)?;
            }

            match backend.wait_key()? {
                Key::Up => {
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

                Key::Down => {
                    if selected < options.len() {
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

                Key::Char('\n') => break Ok(&options[selected]),

                _ => (),
            }
        }
    }
}

fn update_rendered_menu<M, B>(
    backend: &mut B,
    options: &[M],
    prev_selected: usize,
    selected: usize,
    term_size: Coord2D,
) -> io::Result<()>
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
) -> io::Result<()>
where
    M: Menu,
    B: Backend,
{
    let pos = (term_size.x - option.option_name().len() as Coord) / 2;
    backend.goto(Coord2D { x: pos, y: index as Coord })?;
    if selected {
        backend.setfg(Color::Black)?;
        backend.setbg(Color::White)?;
    }
    write!(backend, "{}", option.option_name())?;
    if selected {
        backend.setfg(Color::White)?;
        backend.setbg(Color::Black)?;
    }
    Ok(())
}

fn render_menu<M, B>(
    backend: &mut B,
    options: &[M],
    selected: usize,
    term_size: Coord2D,
) -> io::Result<()>
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
