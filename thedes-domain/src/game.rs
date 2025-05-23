use thedes_geometry::orientation::Direction;
use thiserror::Error;

use crate::{
    block::{Block, PlaceableBlock, SpecialBlock},
    geometry::{CoordPair, Rect},
    map::{AccessError, Map},
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
        Ok(Self { map, player: Player::new(player_position) })
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
}
