mod human;

/// Contains items related to player entities.
pub mod player;

/// Contains items related to NPC (Non-Player Character).
pub mod npc;

/// Contains items related to thedes.
pub mod thede;

/// A biome in the map.
pub mod biome;

/// Language entity related items.
pub mod language;

pub use self::player::Player;
use crate::{
    error::Result,
    graphics::GString,
    math::plane::Camera,
    storage::save::SavedGame,
    terminal,
};

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
    /// This is an NPC entity.
    NPC(npc::Id),
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

            Physical::NPC(id) => {
                game.npcs().load(*id).await?.render(camera, screen).await?;
            },
        }
        Ok(())
    }

    /// Interacts with the player.
    pub async fn interact(
        &self,
        message: &mut GString,
        game: &SavedGame,
    ) -> Result<()> {
        match self {
            Physical::NPC(npc) => {
                let npc = game.npcs().load(*npc).await?;
                npc.interact(message, game).await?;
            },
            _ => (),
        }

        Ok(())
    }
}
