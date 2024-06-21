use thedes_geometry::axis::Direction;

use crate::{
    map::{self, Map},
    player::Player,
};

#[derive(Debug, Clone)]
pub struct Config {
    map_config: map::MapConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self { map_config: map::MapConfig::new() }
    }

    pub fn with_map(self, map_config: map::MapConfig) -> Self {
        Self { map_config, ..self }
    }

    pub fn finish(self) -> Game {
        Game::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    map: Map,
    player: Player,
}

impl Game {
    fn new(config: Config) -> Self {
        let map = config.map_config.finish();
        let player_head = map.rect().top_left + map.rect().size.div_ceil_by(&2);
        let player = Player::new(player_head, Direction::Up);
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
