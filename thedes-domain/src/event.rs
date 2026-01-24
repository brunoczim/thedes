use serde::{Deserialize, Serialize};
use thedes_geometry::orientation::Direction;
use thiserror::Error;

use crate::{
    game::{
        Game,
        MonsterAttackError,
        MonsterFollowError,
        MoveMonsterError,
        SpawnMonsterError,
        VanishMonsterError,
    },
    geometry::Coord,
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
    #[error("Failed to make a monster attack")]
    MonsterAttack(
        #[from]
        #[source]
        MonsterAttackError,
    ),
    #[error("Failed to make a monster follow the player")]
    MonsterFollow(
        #[from]
        #[source]
        MonsterFollowError,
    ),
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Event {
    TrySpawnMonster(MonsterPosition),
    VanishMonster(monster::Id),
    TryMoveMonster(monster::Id, Direction),
    MonsterAttack(monster::Id),
    FollowPlayer { id: monster::Id, period: Coord, limit: u32 },
}

impl Event {
    pub(crate) fn apply(self, game: &mut Game) -> Result<(), ApplyError> {
        match self {
            Self::TrySpawnMonster(position) => {
                game.try_spawn_moster(position)?
            },
            Self::VanishMonster(id) => game.vanish_monster(id)?,
            Self::TryMoveMonster(id, direction) => {
                game.try_move_monster(id, direction)?
            },
            Self::MonsterAttack(id) => game.monster_attack(id)?,
            Self::FollowPlayer { id, period: speed, limit } => {
                game.monster_follow_player(id, speed, limit)?
            },
        }
        Ok(())
    }
}
