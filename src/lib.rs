/// Contains items related to key pressing.
pub mod key;

/// Contains items related to orientation in the game.
pub mod orient;

/// Contains items related to rendering on the screen.
pub mod render;

/// Contains items related to the backend of IO.
pub mod backend;

/// Contains items related to the map of the game.
pub mod map;

/// Contains data related to game sessions (ongoing games).
pub mod session;

/// Contains utilities for timing loops.
pub mod timer;

/// Contains utilities for logging, debugging and crashes.
pub mod log;

use crate::{
    backend::Backend,
    key::Key,
    orient::{Coord2D, Direc},
    render::MIN_SCREEN,
    session::GameSession,
};
use std::{io, time::Duration};

fn check_screen_size<B>(
    backend: &mut B,
    screen_size: &mut Coord2D,
) -> io::Result<bool>
where
    B: Backend,
{
    let mut new_screen = backend.term_size()?;

    if new_screen != *screen_size {
        if new_screen.x < MIN_SCREEN.x || new_screen.y < MIN_SCREEN.y {
            backend.clear_screen()?;
            backend.goto(Coord2D { x: 0, y: 0 })?;
            write!(backend, "RESIZE {:?},{:?}", MIN_SCREEN.x, MIN_SCREEN.y)?;

            while new_screen.x < MIN_SCREEN.x || new_screen.y < MIN_SCREEN.y {
                new_screen = backend.term_size()?
            }
        }

        *screen_size = new_screen;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn game_main<B>() -> io::Result<()>
where
    B: Backend,
{
    let mut backend = B::load()?;
    let mut screen_size = backend.term_size()?;
    let mut session = GameSession::new(screen_size);
    session.render_all(&mut backend)?;
    timer::tick(Duration::from_millis(50), move || {
        if check_screen_size(&mut backend, &mut screen_size)? {
            session.resize_screen(screen_size, &mut backend)?;
        }

        if let Some(key) = backend.try_get_key()? {
            match key {
                Key::Up => session.move_player(Direc::Up, &mut backend)?,
                Key::Down => session.move_player(Direc::Down, &mut backend)?,
                Key::Left => session.move_player(Direc::Left, &mut backend)?,
                Key::Right => {
                    session.move_player(Direc::Right, &mut backend)?
                },
                Key::Char('q') => return Ok(timer::Stop(())),
                _ => (),
            }
        }

        Ok(timer::Continue)
    })
}
