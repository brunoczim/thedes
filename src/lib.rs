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

use crate::{backend::Backend, key::Key, orient::Direc, session::GameSession};
use std::io;

pub fn game_main<B>() -> io::Result<()>
where
    B: Backend,
{
    let mut backend = B::load()?;
    let size = backend.term_size()?;
    let mut session = GameSession::new(size);

    loop {
        let key = backend.wait_key();
        match key? {
            Key::Up => session.move_player(Direc::Up, &mut backend)?,
            Key::Down => session.move_player(Direc::Down, &mut backend)?,
            Key::Left => session.move_player(Direc::Left, &mut backend)?,
            Key::Right => session.move_player(Direc::Right, &mut backend)?,
            Key::Char('q') => break,
            _ => (),
        }
    }

    Ok(())
}
