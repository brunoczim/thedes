/// Iterator extensions.
pub mod iter_ext;

/// Error handling.
pub mod error;

/// Contains utilites for handling uis.
pub mod ui;

/// Contains items related to key pressing.
pub mod key;

/// Contains items related to current player handling.
pub mod player;

/// Contains items related to orientation in the game.
pub mod orient;

/// Contains items related to rendering on the screen.
pub mod render;

/// Contains items related to the backend of IO.
pub mod backend;

/// Terminal handling utilites.
pub mod term;

/// Contains items related to the map of the game.
pub mod map;

/// Contains data related to game sessions (ongoing games).
pub mod session;

/// Storage related functions, such as directories and saved games.
pub mod storage;

use crate::{
    backend::Backend,
    error::GameResult,
    render::Color,
    session::GameSession,
    storage::Save,
    term::Terminal,
    ui::{MainMenu, MainMenuItem, Menu},
};

/// The 'top' function for the game.
pub fn game_main<B>() -> GameResult<()>
where
    B: Backend,
{
    let mut term = Terminal::start(B::load()?)?;
    term.setbg(Color::Black)?;
    term.setfg(Color::White)?;
    term.clear_screen()?;
    term.call(|term| match MainMenu.select(term)? {
        MainMenuItem::NewGame => {
            GameSession::new(term)?.exec(term)?;
            Ok(term::Continue)
        },
        MainMenuItem::LoadGame => {
            if let Some(mut save) = Save::load_from_user(term)? {
                save.session.exec(term)?
            }
            Ok(term::Continue)
        },
        MainMenuItem::Exit => Ok(term::Stop(())),
    })
}
