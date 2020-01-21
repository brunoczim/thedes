/// Thede related items. A thede is a tribe or a people.
pub mod thede;

use crate::{
    block::Block,
    error::GameResult,
    orient::{Camera, Coord2D, Direc},
    storage::save::SavedGame,
    terminal,
};
use std::{
    error::Error,
    fmt::{self, Write},
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
pub struct Id(pub(crate) u32);

impl fmt::Display for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
/// A generic entity type.
pub enum Entity {
    Player(Player),
}

impl Entity {
    /// ID of this entity.
    pub fn id(&self) -> Id {
        match self {
            Self::Player(player) => player.id(),
        }
    }

    /// Render this entity.
    pub async fn render(
        &self,
        camera: Camera,
        term: &mut terminal::Handle,
    ) -> GameResult<()> {
        match self {
            Self::Player(player) => player.render(camera, term).await,
        }
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
/// A generic human entity.
struct Human {
    id: Id,
    head: Coord2D,
    facing: Direc,
}

impl Human {
    /// Coordinates of the pointer of this human.
    fn pointer(&self) -> Coord2D {
        match self.facing {
            Direc::Up => Coord2D { y: self.head.y - 1, ..self.head },
            Direc::Down => Coord2D { y: self.head.y + 1, ..self.head },
            Direc::Left => Coord2D { x: self.head.x - 1, ..self.head },
            Direc::Right => Coord2D { x: self.head.x + 1, ..self.head },
        }
    }

    /// Moves this human in the given direction.
    async fn move_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> GameResult<()> {
        if direc == self.facing {
            self.step(direc, game).await?;
        } else {
            self.turn_around(direc, game).await?;
        }

        Ok(())
    }

    /// Moves this human in the given direction by quick stepping.
    async fn step(&mut self, direc: Direc, game: &SavedGame) -> GameResult<()> {
        let maybe_head = self.head.move_by_direc(direc);
        let maybe_ptr = self.pointer().move_by_direc(direc);
        if let (Some(new_head), Some(new_ptr)) = (maybe_head, maybe_ptr) {
            if self.block_free(new_head, game).await?
                && self.block_free(new_ptr, game).await?
            {
                self.update_head(new_head, game).await?;
            }
        }
        Ok(())
    }

    /// Turns this human around.
    async fn turn_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> GameResult<()> {
        match direc {
            Direc::Up => {
                if let Some(new_y) = self.head.y.checked_sub(1) {
                    let new_coord = Coord2D { y: new_y, ..self.head };
                    if game.block_at(new_coord).await? == Block::Empty {
                        self.update_facing(direc, game).await?;
                    }
                }
            },

            Direc::Down => {
                if let Some(new_y) = self.head.y.checked_add(1) {
                    let new_coord = Coord2D { y: new_y, ..self.head };
                    if game.block_at(new_coord).await? == Block::Empty {
                        self.update_facing(direc, game).await?;
                    }
                }
            },

            Direc::Left => {
                if let Some(new_x) = self.head.x.checked_sub(1) {
                    let new_coord = Coord2D { x: new_x, ..self.head };
                    if game.block_at(new_coord).await? == Block::Empty {
                        self.update_facing(direc, game).await?;
                    }
                }
            },

            Direc::Right => {
                if let Some(new_x) = self.head.x.checked_add(1) {
                    let new_coord = Coord2D { x: new_x, ..self.head };
                    if game.block_at(new_coord).await? == Block::Empty {
                        self.update_facing(direc, game).await?;
                    }
                }
            },
        }

        Ok(())
    }

    /// Renders this human on the screen, with the given sprite.
    pub async fn render<'txtr>(
        &self,
        camera: Camera,
        term: &mut terminal::Handle,
        sprite: HumanSprite<'txtr>,
    ) -> GameResult<()> {
        if let Some(pos) = camera.convert(self.head) {
            term.goto(pos)?;
            term.write_str(sprite.head)?;
        }
        if let Some(pos) = camera.convert(self.pointer()) {
            term.goto(pos)?;
            match self.facing {
                Direc::Up => term.write_str(sprite.up)?,
                Direc::Down => term.write_str(sprite.down)?,
                Direc::Left => term.write_str(sprite.left)?,
                Direc::Right => term.write_str(sprite.right)?,
            }
        }

        Ok(())
    }

    /// Updates the head and the map blocks too.
    async fn update_head(
        &mut self,
        pos: Coord2D,
        game: &SavedGame,
    ) -> GameResult<()> {
        game.update_block_at(self.head, &Block::Empty).await?;
        game.update_block_at(self.pointer(), &Block::Empty).await?;

        self.head = pos;

        let block = Block::Entity(self.id);
        let fut = game.update_block_at(self.pointer(), &block);
        fut.await?;
        let fut = game.update_block_at(self.head, &block);
        fut.await?;
        Ok(())
    }

    /// Updates the facing direction and the map blocks too.
    async fn update_facing(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> GameResult<()> {
        game.update_block_at(self.pointer(), &Block::Empty).await?;

        self.facing = direc;

        game.update_block_at(self.pointer(), &Block::Entity(self.id)).await?;
        Ok(())
    }

    /// Tests if the block is free for moving.
    async fn block_free(
        &self,
        pos: Coord2D,
        game: &SavedGame,
    ) -> GameResult<bool> {
        let block = game.block_at(pos).await?;
        Ok(block == Block::Empty || block == Block::Entity(self.id))
    }
}

#[derive(Debug)]
struct HumanSprite<'string> {
    head: &'string str,
    up: &'string str,
    down: &'string str,
    left: &'string str,
    right: &'string str,
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
    /// Builds player data when initializing the world.
    pub fn new(id: Id) -> Self {
        Self { human: Human { id, head: Coord2D::ORIGIN, facing: Direc::Up } }
    }

    /// Coordinates of the head of the player.
    pub fn head(&self) -> Coord2D {
        self.human.head
    }

    /// Coordinates of the pointer of the player.
    pub fn pointer(&self) -> Coord2D {
        self.human.pointer()
    }

    /// Id of this player.
    pub fn id(&self) -> Id {
        self.human.id
    }

    /// Moves the player in the given direction.
    pub async fn move_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> GameResult<()> {
        self.human.move_around(direc, game).await?;
        game.update_player(self).await?;
        Ok(())
    }

    /// Moves the player in the given direction by quick stepping.
    pub async fn step(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> GameResult<()> {
        self.human.step(direc, game).await?;
        game.update_player(self).await?;
        Ok(())
    }

    /// Renders this player on the screen.
    pub async fn render(
        &self,
        camera: Camera,
        term: &mut terminal::Handle,
    ) -> GameResult<()> {
        let fut = self.human.render(
            camera,
            term,
            HumanSprite {
                head: "O",
                left: "<",
                right: ">",
                down: "V",
                up: "Ʌ",
            },
        );

        fut.await?;

        Ok(())
    }

    /// Renders this player on the screen.
    pub async fn clear(
        &self,
        camera: Camera,
        term: &mut terminal::Handle,
    ) -> GameResult<()> {
        let fut = self.human.render(
            camera,
            term,
            HumanSprite {
                head: " ",
                left: " ",
                right: " ",
                down: " ",
                up: " ",
            },
        );

        fut.await?;

        Ok(())
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
/// Kinds of entities.
pub enum Kind {
    /// A player.
    Player,
}

/// Returns by SaveName::new_game if the game already exists.
#[derive(Debug, Clone, Copy)]
pub struct InvalidId(pub Id);

impl fmt::Display for InvalidId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Invalid entity id {}", self.0)
    }
}

impl Error for InvalidId {}
