use crate::{
    error::GameResult,
    storage::save::{SaveName, SavedGame},
};

#[derive(Debug)]
/// A struct containing everything about the game session.
pub struct Session {
    game: SavedGame,
    name: SaveName,
}

impl Session {
    pub async fn game_loop(&self) -> GameResult<()> {
        Ok(())
    }
}
