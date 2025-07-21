use thedes_geometry::orientation::Direction;
use thiserror::Error;

use crate::{
    game::{Game, MoveMonsterError, SpawnMonsterError, VanishMonsterError},
    monster::{self, MonsterPosition},
};

#[derive(Debug, Error)]
pub enum ApplyError {
    #[error("Failed to spawn a monster")]
    TrySpawnMonster(
        #[from]
        #[source]
        SpawnMonsterError,
    ),
    #[error("Failed to vanish a monster")]
    VanishMonster(
        #[from]
        #[source]
        VanishMonsterError,
    ),
    #[error("Failed to move a monster")]
    TryMoveMonster(
        #[from]
        #[source]
        MoveMonsterError,
    ),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Event {
    TrySpawnMonster(MonsterPosition),
    VanishMonster(monster::Id),
    TryMoveMonster(monster::Id, Direction),
}

impl Event {
    pub fn apply(self, game: &mut Game) -> Result<(), ApplyError> {
        match self {
            Self::TrySpawnMonster(position) => {
                game.try_spawn_moster(position)?
            },
            Self::VanishMonster(id) => game.vanish_monster(id)?,
            Self::TryMoveMonster(id, direction) => {
                game.try_move_monster(id, direction)?
            },
        }
        Ok(())
    }
}
