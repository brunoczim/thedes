use thedes_geometry::axis::Direction;
use thiserror::Error;

use crate::{
    block::Block,
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

#[derive(Debug, Clone)]
pub struct Game {
    map: Map,
    player: Player,
}

impl Game {
    pub fn new(mut map: Map, player: Player) -> Result<Self, CreationError> {
        if let Err(source) = map
            .set_block(player.head(), Block::Player)
            .and_then(|_| map.set_block(player.pointer(), Block::Player))
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
        let Ok(_) = self
            .map
            .rect()
            .checked_move_point_unit(new_head, self.player.facing())
        else {
            return Ok(());
        };
        self.map.unset_block(self.player.head())?;
        self.map.unset_block(self.player.pointer())?;
        self.player.set_head(new_head);
        self.map.set_block(self.player.head(), Block::Player)?;
        self.map.set_block(self.player.pointer(), Block::Player)?;
        Ok(())
    }

    pub fn make_player_face(
        &mut self,
        direction: Direction,
    ) -> Result<(), MovePlayerError> {
        let Ok(_) = self
            .map
            .rect()
            .checked_move_point_unit(self.player.head(), direction)
        else {
            return Ok(());
        };
        self.map.unset_block(self.player.pointer())?;
        self.player.face(direction);
        self.map.set_block(self.player.pointer(), Block::Player)?;
        Ok(())
    }
}
