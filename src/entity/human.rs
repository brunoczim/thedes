use crate::{
    error::Result,
    graphics::{ContrastiveFg, Tile, UpdateColors},
    math::plane::{Camera, Coord2, Direc, Nat},
    matter::Block,
    storage::save::SavedGame,
    terminal,
};

/// A generic human entity.
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
pub struct Human {
    pub head: Coord2<Nat>,
    pub facing: Direc,
}

impl Human {
    /// Coordinates of the pointer of this human.
    pub fn pointer(&self) -> Coord2<Nat> {
        match self.facing {
            Direc::Up => Coord2 { y: self.head.y - 1, ..self.head },
            Direc::Down => Coord2 { y: self.head.y + 1, ..self.head },
            Direc::Left => Coord2 { x: self.head.x - 1, ..self.head },
            Direc::Right => Coord2 { x: self.head.x + 1, ..self.head },
        }
    }

    /// Moves this human in the given direction.
    pub async fn move_around(
        &mut self,
        self_block: &Block,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        if direc == self.facing {
            self.step(self_block, direc, game).await?;
        } else {
            self.turn_around(self_block, direc, game).await?;
        }

        Ok(())
    }

    /// Moves this human in the given direction by quick stepping.
    pub async fn step(
        &mut self,
        self_block: &Block,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        let maybe_head = self.head.move_by_direc(direc);
        let maybe_ptr = self.pointer().move_by_direc(direc);
        if let (Some(new_head), Some(new_ptr)) = (maybe_head, maybe_ptr) {
            if self.block_free(self_block, new_head, game).await?
                && self.block_free(self_block, new_ptr, game).await?
            {
                self.update_head(self_block, new_head, game).await?;
            }
        }
        Ok(())
    }

    /// Turns this human around.
    pub async fn turn_around(
        &mut self,
        self_block: &Block,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        match direc {
            Direc::Up => {
                if let Some(new_y) = self.head.y.checked_sub(1) {
                    let new_coord = Coord2 { y: new_y, ..self.head };
                    if game.blocks().get(new_coord).await? == Block::Empty {
                        self.update_facing(self_block, direc, game).await?;
                    }
                }
            },

            Direc::Down => {
                if let Some(new_y) = self.head.y.checked_add(1) {
                    let new_coord = Coord2 { y: new_y, ..self.head };
                    if game.blocks().get(new_coord).await? == Block::Empty {
                        self.update_facing(self_block, direc, game).await?;
                    }
                }
            },

            Direc::Left => {
                if let Some(new_x) = self.head.x.checked_sub(1) {
                    let new_coord = Coord2 { x: new_x, ..self.head };
                    if game.blocks().get(new_coord).await? == Block::Empty {
                        self.update_facing(self_block, direc, game).await?;
                    }
                }
            },

            Direc::Right => {
                if let Some(new_x) = self.head.x.checked_add(1) {
                    let new_coord = Coord2 { x: new_x, ..self.head };
                    if game.blocks().get(new_coord).await? == Block::Empty {
                        self.update_facing(self_block, direc, game).await?;
                    }
                }
            },
        }

        Ok(())
    }

    /// Renders this human on the screen, with the given sprite.
    pub async fn render<'guard, S>(
        &self,
        camera: Camera,
        screen: &mut terminal::Screen<'guard>,
        sprite: &S,
    ) -> Result<()>
    where
        S: Sprite,
    {
        if let Some(pos) = camera.convert(self.head) {
            screen.set(pos, sprite.head());
        }
        if let Some(pos) = camera.convert(self.pointer()) {
            let tile = match self.facing {
                Direc::Up => sprite.up(),
                Direc::Down => sprite.down(),
                Direc::Left => sprite.left(),
                Direc::Right => sprite.right(),
            };
            screen.set(pos, tile);
        }

        Ok(())
    }

    /// Updates the head and the map blocks too.
    pub async fn update_head(
        &mut self,
        self_block: &Block,
        pos: Coord2<Nat>,
        game: &SavedGame,
    ) -> Result<()> {
        game.blocks().set(self.head, &Block::Empty).await?;
        game.blocks().set(self.pointer(), &Block::Empty).await?;

        self.head = pos;

        let fut = game.blocks().set(self.pointer(), self_block);
        fut.await?;
        let fut = game.blocks().set(self.head, self_block);
        fut.await?;
        Ok(())
    }

    /// Updates the facing direction and the map blocks too.
    pub async fn update_facing(
        &mut self,
        self_block: &Block,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        game.blocks().set(self.pointer(), &Block::Empty).await?;

        self.facing = direc;

        game.blocks().set(self.pointer(), self_block).await?;
        Ok(())
    }

    /// Tests if the block is free for moving.
    pub async fn block_free(
        &self,
        self_block: &Block,
        pos: Coord2<Nat>,
        game: &SavedGame,
    ) -> Result<bool> {
        let block = game.blocks().get(pos).await?;
        Ok(block == Block::Empty || block == *self_block)
    }
}

/// The sprite of a human.
pub trait Sprite {
    /// Semi-tile for the head.
    fn head(&self) -> Tile<ContrastiveFg>;

    /// Semi-tile for the up pointer.
    fn up(&self) -> Tile<ContrastiveFg>;

    /// Semi-tile for the down pointer.
    fn down(&self) -> Tile<ContrastiveFg>;

    /// Semi-tile for the left pointer.
    fn left(&self) -> Tile<ContrastiveFg>;

    /// Semi-tile for the right pointer.
    fn right(&self) -> Tile<ContrastiveFg>;
}
