/// Iterator extensions.
pub mod iter_ext;

/// Random number generation utilites.
pub mod rand;

/// Error handling.
pub mod error;

/// Contains items related to terminal handling.
pub mod terminal;

/// Contains items related to orientation in the game.
pub mod orient;

/// Contains items related to rendering on the screen.
pub mod render;

/// Contains items related to user input.
pub mod input;

/// Storage related functions, such as directories and saved games.
pub mod storage;

/*
/// Contains utilites for handling uis.
pub mod ui;

/// Contains items related to current player handling.
pub mod player;

/// Terminal handling utilites.
pub mod term;

/// Contains items related to the map of the game.
pub mod map;

/// Contains data related to game sessions (ongoing games).
pub mod session;

*/
use crate::{
    error::GameResult,
    orient::Coord2D,
    render::{Color, MIN_SCREEN},
};

/// The 'top' function for the game.
pub async fn game_main() -> GameResult<()> {
    let mut term = terminal::Handle::new().await?;
    term.set_bg(Color::Black)?;
    term.set_fg(Color::White)?;
    term.clear_screen()?;
    term.flush().await?;

    let fut = term.on_resize(|mut term, evt| async move {
        if evt.size.x < MIN_SCREEN.x || evt.size.y < MIN_SCREEN.y {
            let fut = term.set_screen_size(Coord2D::from_map(|axis| {
                evt.size[axis].min(MIN_SCREEN[axis])
            }));
            fut.await?;
        }

        Ok(())
    });
    fut.await;

    term.async_drop().await?;
    Ok(())
}
