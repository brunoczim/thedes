use crate::{
    error::GameResult,
    storage::save::{SaveName, SavedGame},
    terminal,
};

#[derive(Debug)]
/// A struct containing everything about the game session.
pub struct Session {
    game: SavedGame,
    name: SaveName,
}

impl Session {
    pub fn new(game: SavedGame, name: SaveName) -> Self {
        Self { game, name }
    }

    pub async fn game_loop(
        &self,
        term: &mut terminal::Handle,
    ) -> GameResult<()> {
        match term.listen_event().await {
            _ => unimplemented!(),
        }
    }
}
