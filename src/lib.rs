#![recursion_limit = "256"]
#![deny(unused_must_use)]

/// Asynchronous detaching utilities.
pub mod detach;

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

/// Contains utilites for handling uis.
pub mod ui;

/*
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
    ui::{menu_select, MainMenu, MainMenuItem},
};

/// The 'top' function for the game.
pub async fn game_main() -> GameResult<()> {
    let mut term = terminal::Handle::new().await?;

    loop {
        match menu_select(&MainMenu, &mut term).await? {
            MainMenuItem::NewGame => {},
            MainMenuItem::LoadGame => {},
            MainMenuItem::Exit => break Ok(()),
        }
    }
}
