use std::collections::HashMap;

use thedes_geometry::orientation::Direction;
use thiserror::Error;

use crate::{
    block::{Block, PlaceableBlock, SpecialBlock},
    event::{self, Event},
    geometry::{CoordPair, Rect},
    map::{AccessError, Map},
    monster::{self, IdShortageError, Monster, MonsterPosition},
    player::{Player, PlayerPosition},
    stat::StatValue,
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

#[derive(Debug, Error)]
pub enum MonsterAttackError {
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
pub enum MonsterFollowError {
    #[error("Invalid monster ID")]
    InvalidId(
        #[from]
        #[source]
        monster::InvalidId,
    ),
    #[error("Failed to move monster")]
    MoveMonster(#[from] MoveMonsterError),
}

fn blocks_movement(block: Block, this: SpecialBlock) -> bool {
    match block {
        Block::Placeable(block) => block != PlaceableBlock::Air,
        Block::Special(block) => {
            block != this
                && matches!(
                    block,
                    SpecialBlock::Player | SpecialBlock::Monster(_)
                )
        },
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    map: Map,
    player: Player,
    monster_registry: monster::Registry,
    event_schedule: HashMap<u64, Vec<Event>>,
    event_epoch: u64,
}

impl Game {
    pub fn new(mut map: Map, player: Player) -> Result<Self, InitError> {
        if let Err(source) = map
            .set_block(player.position().head(), SpecialBlock::Player)
            .and_then(|_| {
                map.set_block(player.position().pointer(), SpecialBlock::Player)
            })
        {
            return Err(InitError::PlayerOutsideMap {
                map_rect: map.rect(),
                player_position: player.position().clone(),
                source,
            });
        }
        Ok(Self {
            map,
            player,
            monster_registry: monster::Registry::new(),
            event_schedule: HashMap::new(),
            event_epoch: 0,
        })
    }

    pub fn schedule_event(&mut self, event: Event, event_ticks: u32) {
        let event_ticks = u64::from(event_ticks) + self.event_epoch;
        self.event_schedule.entry(event_ticks).or_default().push(event);
    }

    pub fn execute_events(&mut self) -> Result<(), event::ApplyError> {
        let old_epoch = self.event_epoch;
        self.event_epoch += 1;
        for event in
            self.event_schedule.remove(&old_epoch).into_iter().flatten()
        {
            event.apply(self)?;
        }
        Ok(())
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
        if blocks_movement(self.map.get_block(new_head)?, SpecialBlock::Player)
        {
            return Ok(());
        }
        if blocks_movement(
            self.map.get_block(new_pointer)?,
            SpecialBlock::Player,
        ) {
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
        if blocks_movement(self.map.get_block(new_head)?, SpecialBlock::Player)
        {
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
        if blocks_movement(
            self.map.get_block(new_body)?,
            SpecialBlock::Monster(id),
        ) {
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

    pub fn monster_attack(
        &mut self,
        id: monster::Id,
    ) -> Result<(), MonsterAttackError> {
        let monster = self.monster_registry.get_by_id(id)?;
        let Some(next_block) = monster
            .position()
            .body()
            .checked_move_unit(monster.position().facing())
        else {
            return Ok(());
        };
        let Ok(block) = self.map.get_block(next_block) else {
            return Ok(());
        };
        match block {
            Block::Special(SpecialBlock::Player) => {
                self.player.damage(1);
            },
            _ => (),
        }
        Ok(())
    }

    pub fn monster_follow_player(
        &mut self,
        id: monster::Id,
        limit: u32,
    ) -> Result<(), MonsterFollowError> {
        let monster = self.monster_registry.get_by_id(id)?;

        let (_, Some(vec)) = self
            .player()
            .position()
            .head()
            .diagonal_direction_from(monster.position().body())
            .max_with_axis_by_key(|opt| opt.map(|vec| vec.magnitude))
        else {
            return Ok(());
        };

        self.try_move_monster(id, vec.direction)?;

        if let Some(new_limit) = limit.checked_sub(1) {
            self.schedule_event(
                Event::FollowPlayer { id, limit: new_limit },
                150,
            );
        }
        Ok(())
    }

    pub fn damage_player(&mut self, amount: StatValue) {
        self.player.damage(amount);
    }

    pub fn heal_player(&mut self, amount: StatValue) {
        self.player.heal(amount);
    }
}
