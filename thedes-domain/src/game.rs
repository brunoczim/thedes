use thedes_geometry::orientation::Direction;
use thiserror::Error;

use crate::{
    block::{Block, PlaceableBlock, SpecialBlock},
    geometry::{CoordPair, Rect},
    map::{AccessError, Map},
    monster::{self, IdShortageError, Monster, MonsterPosition},
    player::{Player, PlayerPosition},
};

#[derive(Debug, Error)]
pub enum InitError {
    #[error(
        "Player with head {} and pointer {} is outside of map {map_rect}",
        .player_position.head(),
        .player_position.pointer(),
    )]
    PlayerOutsideMap {
        map_rect: Rect,
        player_position: PlayerPosition,
        source: AccessError,
    },
}

#[derive(Debug, Error)]
pub enum MovePlayerError {
    #[error("Failed to access player positions")]
    Access(
        #[from]
        #[source]
        AccessError,
    ),
}

#[derive(Debug, Error)]
pub enum SpawnMonsterError {
    #[error("Failed to access map location")]
    MapAccess(
        #[from]
        #[source]
        AccessError,
    ),
    #[error("Run out of identifiers")]
    IdShortage(
        #[from]
        #[source]
        IdShortageError,
    ),
}

#[derive(Debug, Error)]
pub enum VanishMonsterError {
    #[error("Invalid monster ID")]
    InvalidId(
        #[from]
        #[source]
        monster::InvalidId,
    ),
    #[error("Failed to access map location")]
    MapAccess(
        #[from]
        #[source]
        AccessError,
    ),
}

#[derive(Debug, Error)]
pub enum MoveMonsterError {
    #[error("Invalid monster ID")]
    InvalidId(
        #[from]
        #[source]
        monster::InvalidId,
    ),
    #[error("Failed to access map location")]
    MapAccess(
        #[from]
        #[source]
        AccessError,
    ),
}

fn blocks_movement(block: Block) -> bool {
    match block {
        Block::Placeable(block) => block != PlaceableBlock::Air,
        Block::Special(block) => {
            !matches!(block, SpecialBlock::Player | SpecialBlock::Monster(_))
        },
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    map: Map,
    player: Player,
    monster_registry: monster::Registry,
}

impl Game {
    pub fn new(
        mut map: Map,
        player_position: PlayerPosition,
    ) -> Result<Self, InitError> {
        if let Err(source) = map
            .set_block(player_position.head(), SpecialBlock::Player)
            .and_then(|_| {
                map.set_block(player_position.pointer(), SpecialBlock::Player)
            })
        {
            return Err(InitError::PlayerOutsideMap {
                map_rect: map.rect(),
                player_position,
                source,
            });
        }
        Ok(Self {
            map,
            player: Player::new(player_position),
            monster_registry: monster::Registry::new(),
        })
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn place_block(
        &mut self,
        point: CoordPair,
        block: PlaceableBlock,
    ) -> Result<(), AccessError> {
        self.map.set_placeable_block(point, block)
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn move_player_pointer(
        &mut self,
        direction: Direction,
    ) -> Result<(), MovePlayerError> {
        if self.player.position().facing() == direction {
            self.move_player_head(direction)?;
        } else {
            self.make_player_face(direction)?;
        }
        Ok(())
    }

    pub fn move_player_head(
        &mut self,
        direction: Direction,
    ) -> Result<(), MovePlayerError> {
        let Ok(new_head) = self
            .map
            .rect()
            .checked_move_point_unit(self.player.position().head(), direction)
        else {
            return Ok(());
        };
        let Ok(new_pointer) = self
            .map
            .rect()
            .checked_move_point_unit(new_head, self.player.position().facing())
        else {
            return Ok(());
        };
        if blocks_movement(self.map.get_block(new_head)?) {
            return Ok(());
        }
        if blocks_movement(self.map.get_block(new_pointer)?) {
            return Ok(());
        }
        self.map
            .set_block(self.player.position().head(), PlaceableBlock::Air)?;
        self.map
            .set_block(self.player.position().pointer(), PlaceableBlock::Air)?;
        self.player.position_mut().set_head(new_head);
        self.map
            .set_block(self.player.position().head(), SpecialBlock::Player)?;
        self.map.set_block(
            self.player.position().pointer(),
            SpecialBlock::Player,
        )?;
        Ok(())
    }

    pub fn make_player_face(
        &mut self,
        direction: Direction,
    ) -> Result<(), MovePlayerError> {
        let Ok(new_head) = self
            .map()
            .rect()
            .checked_move_point_unit(self.player.position().head(), direction)
        else {
            return Ok(());
        };
        if blocks_movement(self.map.get_block(new_head)?) {
            return Ok(());
        }
        self.map
            .set_block(self.player.position().pointer(), PlaceableBlock::Air)?;
        self.player.position_mut().face(direction);
        self.map.set_block(
            self.player.position().pointer(),
            SpecialBlock::Player,
        )?;
        Ok(())
    }

    pub fn monster_registry(&self) -> &monster::Registry {
        &self.monster_registry
    }

    pub fn try_spawn_moster(
        &mut self,
        pos: MonsterPosition,
    ) -> Result<(), SpawnMonsterError> {
        let block_value = self.map().get_block(pos.body())?;
        if block_value == Block::Placeable(PlaceableBlock::Air) {
            let monster = Monster::new(pos);
            let monster_id = self.monster_registry.create_as(monster)?;
            self.map
                .set_block(pos.body(), SpecialBlock::Monster(monster_id))?;
        }
        Ok(())
    }

    pub fn vanish_monster(
        &mut self,
        id: monster::Id,
    ) -> Result<(), VanishMonsterError> {
        let monster = self.monster_registry.remove(id)?;
        self.map.set_block(monster.position().body(), PlaceableBlock::Air)?;
        Ok(())
    }

    pub fn try_move_monster(
        &mut self,
        id: monster::Id,
        direction: Direction,
    ) -> Result<(), MoveMonsterError> {
        let pos = self.monster_registry.get_by_id(id)?.position();
        if pos.facing() == direction {
            self.move_monster_head(id, direction)?;
        } else {
            self.make_monster_face(id, direction)?;
        }
        Ok(())
    }

    pub fn move_monster_head(
        &mut self,
        id: monster::Id,
        direction: Direction,
    ) -> Result<(), MoveMonsterError> {
        let pos = self.monster_registry.get_by_id(id)?.position();
        let Ok(new_body) =
            self.map.rect().checked_move_point_unit(pos.body(), direction)
        else {
            return Ok(());
        };
        if blocks_movement(self.map.get_block(new_body)?) {
            return Ok(());
        }
        self.map.set_block(pos.body(), PlaceableBlock::Air)?;
        self.monster_registry
            .get_by_id_mut(id)?
            .position_mut()
            .set_body(new_body);
        self.map.set_block(new_body, SpecialBlock::Monster(id))?;
        Ok(())
    }

    pub fn make_monster_face(
        &mut self,
        id: monster::Id,
        direction: Direction,
    ) -> Result<(), MoveMonsterError> {
        self.monster_registry.get_by_id_mut(id)?.position_mut().face(direction);
        Ok(())
    }
}
