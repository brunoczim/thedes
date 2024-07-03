use thedes_geometry::axis::Direction;

use crate::{map::Map, player::Player};

#[derive(Debug, Clone)]
pub struct Game {
    map: Map,
    player: Player,
}

impl Game {
    pub(crate) fn new(map: Map, player: Player) -> Self {
        Self { map, player }
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
