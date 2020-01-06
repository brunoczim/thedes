use crate::{
    entity,
    error::GameResult,
    orient::{Camera, Coord2D},
    render::MIN_SCREEN,
    storage::save::{SaveName, SavedGame},
    terminal,
};

#[derive(Debug)]
/// A struct containing everything about the game session.
pub struct Session {
    game: SavedGame,
    name: SaveName,
    player: entity::Player,
    camera: Camera,
}

impl Session {
    /// Initializes a new session from given saved game and save name.
    pub async fn new(game: SavedGame, name: SaveName) -> GameResult<Self> {
        let player_id = game.player_id().await?;
        let player = game.player(player_id).await?;
        Ok(Self {
            game,
            name,
            // dummy camera
            camera: Camera::new(player.head(), MIN_SCREEN),
            player,
        })
    }

    /// The main loop of the game.
    pub async fn game_loop(
        &mut self,
        term: &mut terminal::Handle,
    ) -> GameResult<()> {
        self.update_camera(term.screen_size());
        self.render(term).await?;
        loop {
            match term.listen_event().await {
                _ => unimplemented!(),
            }
        }
    }

    async fn render(&self, term: &mut terminal::Handle) -> GameResult<()> {
        self.player.render(&self.camera, term).await?;
        Ok(())
    }

    /// Updates the camera acording to the available size.
    fn update_camera(&mut self, screen_size: Coord2D) {
        self.camera = Camera::new(self.player.head(), screen_size);
    }
}
