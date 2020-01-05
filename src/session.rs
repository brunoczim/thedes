use crate::storage::save::{SaveName, SavedGame};

#[derive(Debug)]
/// A struct containing everything about the game session.
pub struct Session {
    game: SavedGame,
    name: SaveName,
}
