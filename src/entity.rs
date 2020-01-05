use crate::{
    block::Block,
    error::GameResult,
    orient::{Coord2D, Direc},
    storage::save::SavedGame,
};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
/// An entity ID.
pub struct Id(u32);

impl Id {
    /// Human's ID.
    pub const PLAYER: Self = Self(0);
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
/// A human entity.
pub struct Human {
    center: Coord2D,
    facing: Direc,
}

impl Human {
    /// Moves this human in the given direction.
    pub async fn move_direc(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> GameResult<()> {
        if direc == self.facing {
            match self.facing {
                Direc::Up => {
                    if let Some(new_y) = self.center.y.checked_sub(2) {
                        let new_coord = Coord2D { y: new_y, ..self.center };
                        if game.block_at(new_coord).await? == Block::Empty {
                            self.center.y += 1;
                        }
                    }
                },

                Direc::Down => {
                    if let Some(new_y) = self.center.y.checked_add(2) {
                        let new_coord = Coord2D { y: new_y, ..self.center };
                        if game.block_at(new_coord).await? == Block::Empty {
                            self.center.y += 1;
                        }
                    }
                },

                Direc::Left => {
                    if let Some(newx) = self.center.x.checked_sub(2) {
                        let new_coord = Coord2D { x: newx, ..self.center };
                        if game.block_at(new_coord).await? == Block::Empty {
                            self.center.x += 1;
                        }
                    }
                },

                Direc::Right => {
                    if let Some(newx) = self.center.x.checked_add(2) {
                        let new_coord = Coord2D { x: newx, ..self.center };
                        if game.block_at(new_coord).await? == Block::Empty {
                            self.center.x += 1;
                        }
                    }
                },
            }

            Ok(())
        } else {
            self.turn_around(direc, game).await
        }
    }

    /// Turns this human around.
    pub async fn turn_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> GameResult<()> {
        match direc {
            Direc::Up => {
                if let Some(new_y) = self.center.y.checked_sub(1) {
                    let new_coord = Coord2D { y: new_y, ..self.center };
                    if game.block_at(new_coord).await? == Block::Empty {
                        self.facing = direc;
                    }
                }
            },

            Direc::Down => {
                if let Some(new_y) = self.center.y.checked_add(1) {
                    let new_coord = Coord2D { y: new_y, ..self.center };
                    if game.block_at(new_coord).await? == Block::Empty {
                        self.facing = direc;
                    }
                }
            },

            Direc::Left => {
                if let Some(new_x) = self.center.x.checked_sub(1) {
                    let new_coord = Coord2D { x: new_x, ..self.center };
                    if game.block_at(new_coord).await? == Block::Empty {
                        self.facing = direc;
                    }
                }
            },

            Direc::Right => {
                if let Some(new_x) = self.center.x.checked_add(1) {
                    let new_coord = Coord2D { x: new_x, ..self.center };
                    if game.block_at(new_coord).await? == Block::Empty {
                        self.facing = direc;
                    }
                }
            },
        }
        Ok(())
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
/// Player entity.
pub struct Player {
    human: Human,
}

impl Player {
    /// Player data when initializing the world.
    pub const INIT: Self =
        Self { human: Human { center: Coord2D::ORIGIN, facing: Direc::Up } };
}
