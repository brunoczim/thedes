use thedes_geometry::Direction;
use thiserror::Error;

pub type Coord = u16;

pub type CoordPair = thedes_geometry::CoordPair<Coord>;

pub type Rect = thedes_geometry::Rect<Coord>;

#[derive(Debug, Error)]
#[error("Point is outside of map")]
pub struct InvalidMapPoint {
    #[from]
    source: thedes_geometry::HorzAreaError<usize>,
}

#[derive(Debug, Error)]
pub enum InvalidMapRect {
    #[error("Map size {given_size} is below the minimum of {}", Map::MIN_SIZE)]
    TooSmall { given_size: CoordPair },
    #[error("Map rectangle {given_rect} has overflowing bottom right point")]
    BottomRightOverflow { given_rect: Rect },
    #[error("Map rectangle size {given_size} has overflowing area")]
    AreaOverflow { given_size: CoordPair },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ground {
    Grass,
    Sand,
    Stone,
}

impl Default for Ground {
    fn default() -> Self {
        Self::Grass
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Map {
    rect: Rect,
    ground_layer: Box<[Ground]>,
}

impl Map {
    pub const MIN_SIZE: CoordPair = CoordPair { x: 64, y: 64 };

    pub fn new(rect: Rect) -> Result<Self, InvalidMapRect> {
        if rect.size.zip2(Self::MIN_SIZE).any(|(given, min)| given < min) {
            Err(InvalidMapRect::TooSmall { given_size: rect.size })?
        }
        if rect.checked_bottom_right().is_none() {
            Err(InvalidMapRect::BottomRightOverflow { given_rect: rect })?
        }
        let size = rect
            .map(usize::from)
            .checked_total_area()
            .ok_or(InvalidMapRect::AreaOverflow { given_size: rect.size })?;
        Ok(Self {
            rect,
            ground_layer: Box::from(vec![Ground::default(); size]),
        })
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn get_ground(
        &self,
        point: CoordPair,
    ) -> Result<Ground, InvalidMapPoint> {
        let index = self.to_flat_index(point)?;
        Ok(self.ground_layer[index])
    }

    pub fn set_ground(
        &mut self,
        point: CoordPair,
        value: Ground,
    ) -> Result<(), InvalidMapPoint> {
        let index = self.to_flat_index(point)?;
        self.ground_layer[index] = value;
        Ok(())
    }

    fn to_flat_index(
        &self,
        point: CoordPair,
    ) -> Result<usize, InvalidMapPoint> {
        let index = self
            .rect
            .map(usize::from)
            .checked_horz_area_up_to(point.map(usize::from))?;
        Ok(index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Player {
    head: CoordPair,
    facing: Direction,
}

impl Player {
    pub fn head(&self) -> CoordPair {
        self.head
    }

    pub fn facing(&self) -> Direction {
        self.facing
    }

    pub fn pointer(&self) -> CoordPair {
        self.head.move_unit(self.facing)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Game {
    map: Map,
    player: Player,
}

impl Game {
    pub fn new(map: Map) -> Self {
        let player = Player {
            head: map.rect.top_left + map.rect.size.div_ceil_by(&2),
            facing: Direction::Up,
        };
        Self { map, player }
    }

    pub fn move_player_pointer(&mut self, direction: Direction) {
        if self.player.facing == direction {
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
        self.player.head = new_head;
    }

    pub fn make_player_face(&mut self, direction: Direction) {
        let Ok(_) = self
            .map
            .rect()
            .checked_move_point_unit(self.player.head(), direction)
        else {
            return;
        };
    }

    pub fn player(&self) -> &Player {
        &self.player
    }
}
