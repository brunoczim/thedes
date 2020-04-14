mod human;

/// Contains items related to player entities.
pub mod player;

pub use self::player::Player;
use crate::{coord::Camera, error::Result, storage::save::SavedGame, terminal};

/// Union of all the entities with physical form.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Physical {
    /// This is a player entity.
    Player(player::Id),
}

impl Physical {
    /// Renders this entity on the screen.
    pub async fn render<'guard>(
        &self,
        camera: Camera,
        screen: &mut terminal::Screen<'guard>,
        game: &SavedGame,
    ) -> Result<()> {
        match self {
            Physical::Player(id) => {
                game.players().load(*id).await?.render(camera, screen).await?;
            },
        }
        Ok(())
    }
}
