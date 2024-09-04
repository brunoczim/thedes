use thedes_geometry::axis::Direction;
use thiserror::Error;

use crate::{
    block::{Block, PlaceableBlock, SpecialBlock},
    geometry::Rect,
    map::{AccessError, Map},
    player::Player,
};

#[derive(Debug, Error)]
pub enum CreationError {
    #[error(
        "Player with head {} and pointer {} is outside of map {map_rect}",
        .player.head(),
        .player.pointer(),
    )]
    PlayerOutsideMap { map_rect: Rect, player: Player, source: AccessError },
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

fn block_allows_player_move(block: Block) -> bool {
    match block {
        Block::Placeable(block) => block == PlaceableBlock::Air,
        Block::Special(block) => block == SpecialBlock::Player,
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    map: Map,
    player: Player,
}

impl Game {
    pub fn new(mut map: Map, player: Player) -> Result<Self, CreationError> {
        if let Err(source) = map
            .set_block(player.head(), SpecialBlock::Player)
            .and_then(|_| map.set_block(player.pointer(), SpecialBlock::Player))
        {
            return Err(CreationError::PlayerOutsideMap {
                map_rect: map.rect(),
                player,
                source,
            });
        }
        Ok(Self { map, player })
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn move_player_pointer(
        &mut self,
        direction: Direction,
    ) -> Result<(), MovePlayerError> {
        if self.player.facing() == direction {
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
            .checked_move_point_unit(self.player.head(), direction)
        else {
            return Ok(());
        };
        let Ok(new_pointer) = self
            .map
            .rect()
            .checked_move_point_unit(new_head, self.player.facing())
        else {
            return Ok(());
        };
        if block_allows_player_move(self.map.get_block(new_head)?) {
            return Ok(());
        }
        if block_allows_player_move(self.map.get_block(new_pointer)?) {
            return Ok(());
        }
        self.map.set_block(self.player.head(), PlaceableBlock::Air)?;
        self.map.set_block(self.player.pointer(), PlaceableBlock::Air)?;
        self.player.set_head(new_head);
        self.map.set_block(self.player.head(), SpecialBlock::Player)?;
        self.map.set_block(self.player.pointer(), SpecialBlock::Player)?;
        Ok(())
    }

    pub fn make_player_face(
        &mut self,
        direction: Direction,
    ) -> Result<(), MovePlayerError> {
        let Ok(new_head) = self
            .map()
            .rect()
            .checked_move_point_unit(self.player.head(), direction)
        else {
            return Ok(());
        };
        if block_allows_player_move(self.map.get_block(new_head)?) {
            return Ok(());
        }
        self.map.set_block(self.player.pointer(), PlaceableBlock::Air)?;
        self.player.face(direction);
        self.map.set_block(self.player.pointer(), SpecialBlock::Player)?;
        Ok(())
    }
}
