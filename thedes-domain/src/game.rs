use thedes_geometry::axis::Direction;
use thiserror::Error;

use crate::{
    block::{Block, PlaceableBlock, SpecialBlock},
    geometry::{CoordPair, Rect},
    item::{self, Inventory, SlotEntry},
    map::{AccessError, Map},
    player::{Player, PlayerPosition},
    time::Time,
};

#[derive(Debug, Error)]
pub enum CreationError {
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

fn blocks_player_move(block: Block) -> bool {
    match block {
        Block::Placeable(block) => block != PlaceableBlock::Air,
        Block::Special(block) => block != SpecialBlock::Player,
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    map: Map,
    player: Player,
    time: Time,
}

impl Game {
    pub fn new(
        mut map: Map,
        player_position: PlayerPosition,
    ) -> Result<Self, CreationError> {
        if let Err(source) = map
            .set_block(player_position.head(), SpecialBlock::Player)
            .and_then(|_| {
                map.set_block(player_position.pointer(), SpecialBlock::Player)
            })
        {
            return Err(CreationError::PlayerOutsideMap {
                map_rect: map.rect(),
                player_position,
                source,
            });
        }
        Ok(Self {
            map,
            player: Player::new(player_position, Inventory::new()),
            time: Time::new(),
        })
    }

    pub fn on_post_tick(&mut self) {
        self.time.on_tick();
    }

    pub fn time(&self) -> &Time {
        &self.time
    }

    pub fn set_time(&mut self, stamp: u64) {
        self.time.set(stamp);
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
        if blocks_player_move(self.map.get_block(new_head)?) {
            return Ok(());
        }
        if blocks_player_move(self.map.get_block(new_pointer)?) {
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
        if blocks_player_move(self.map.get_block(new_head)?) {
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

    pub fn player_picked(
        &mut self,
        slot_index: usize,
        entry: SlotEntry,
    ) -> Result<(), item::AccessError> {
        self.player.inventory_mut().set(slot_index, entry)
    }
}
