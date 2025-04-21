use thedes_domain::game::Game;

#[derive(Debug, Clone)]
pub struct Session {
    game: Game,
}

impl Session {
    pub fn new(game: Game) -> Self {
        Self { game }
    }
}
