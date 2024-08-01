use thedes_geometry::axis::Direction;
use thiserror::Error;

use crate::{geometry::Rect, map::Map, player::Player};

#[derive(Debug, Error)]
pub enum CreationError {
    #[error(
        "Player with head {} and pointer {} is outside of map {map_rect}",
        .player.head(),
        .player.pointer(),
    )]
    PlayerOutsideMap { map_rect: Rect, player: Player },
}

#[derive(Debug, Clone)]
pub struct Game {
    map: Map,
    player: Player,
}

impl Game {
    pub fn new(map: Map, player: Player) -> Result<Self, CreationError> {
        if !map.rect().contains_point(player.head())
            || !map.rect().contains_point(player.pointer())
        {
            return Err(CreationError::PlayerOutsideMap {
                map_rect: map.rect(),
                player,
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

    pub fn move_player_pointer(&mut self, direction: Direction) {
        if self.player.facing() == direction {
            self.move_player_head(direction);
        } else {
            self.make_player_face(direction);
        }
    }

    pub fn move_player_head(&mut self, direction: Direction) {
        let Ok(new_head) = self
            .map
            .rect()
            .checked_move_point_unit(self.player.head(), direction)
        else {
            return;
        };
        let Ok(_) = self
            .map
            .rect()
            .checked_move_point_unit(new_head, self.player.facing())
        else {
            return;
        };
        self.player.set_head(new_head);
    }

    pub fn make_player_face(&mut self, direction: Direction) {
        let Ok(_) = self
            .map
            .rect()
            .checked_move_point_unit(self.player.head(), direction)
        else {
            return;
        };
        self.player.face(direction);
    }
}
