use crate::{
    error::Result,
    graphics::Foreground,
    math::plane::{Camera, Coord2, Direc, Nat},
    matter::Block,
    storage::save::SavedGame,
    terminal,
};

pub type Health = u8;

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
    /// Coordinates of the head.
    pub head: Coord2<Nat>,
    /// The direction the human is facing.
    pub facing: Direc,
    /// The human health.
    pub health: Health,
    /// The human maximum health.
    pub max_health: Health,
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
        self_block: Block,
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
        self_block: Block,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        let maybe_head = self.head.move_by_direc(direc);
        let maybe_ptr = self.pointer().move_by_direc(direc);
        if let (Some(new_head), Some(new_ptr)) = (maybe_head, maybe_ptr) {
            if self.block_free(&self_block, new_head, game).await?
                && self.block_free(&self_block, new_ptr, game).await?
            {
                self.update_head(self_block, new_head, game).await?;
            }
        }
        Ok(())
    }

    /// Turns this human around.
    pub async fn turn_around(
        &mut self,
        self_block: Block,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        let new_coord = match direc {
            Direc::Up => self
                .head
                .y
                .checked_sub(1)
                .map(|new_y| Coord2 { y: new_y, ..self.head }),

            Direc::Down => self
                .head
                .y
                .checked_add(1)
                .map(|new_y| Coord2 { y: new_y, ..self.head }),

            Direc::Left => self
                .head
                .x
                .checked_sub(1)
                .map(|new_x| Coord2 { x: new_x, ..self.head }),

            Direc::Right => self
                .head
                .x
                .checked_add(1)
                .map(|new_x| Coord2 { x: new_x, ..self.head }),
        };

        if let Some(new_coord) = new_coord {
            let empty = game.map().block(new_coord).await? == Block::Empty;
            if empty {
                self.update_facing(self_block, direc, game).await?;
            }
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
            let bg = screen.get(pos).colors.bg;
            let fg = sprite.head();
            screen.set(pos, fg.make_tile(bg));
        }
        if let Some(pos) = camera.convert(self.pointer()) {
            let bg = screen.get(pos).colors.bg;
            let fg = match self.facing {
                Direc::Up => sprite.up(),
                Direc::Down => sprite.down(),
                Direc::Left => sprite.left(),
                Direc::Right => sprite.right(),
            };
            screen.set(pos, fg.make_tile(bg));
        }

        Ok(())
    }

    /// Updates the head and the map blocks too.
    pub async fn update_head(
        &mut self,
        self_block: Block,
        pos: Coord2<Nat>,
        game: &SavedGame,
    ) -> Result<()> {
        game.map().set_block(self.head, Block::Empty).await?;
        game.map().set_block(self.pointer(), Block::Empty).await?;

        self.head = pos;

        game.map().set_block(self.head, self_block.clone()).await?;
        game.map().set_block(self.pointer(), self_block).await?;
        Ok(())
    }

    /// Updates the facing direction and the map blocks too.
    pub async fn update_facing(
        &mut self,
        self_block: Block,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        game.map().set_block(self.pointer(), Block::Empty).await?;
        self.facing = direc;
        game.map().set_block(self.pointer(), self_block).await?;
        Ok(())
    }

    /// Tests if the block is free for moving.
    pub async fn block_free(
        &self,
        self_block: &Block,
        pos: Coord2<Nat>,
        game: &SavedGame,
    ) -> Result<bool> {
        let block = game.map().block(pos).await?;
        Ok(block == Block::Empty || block == *self_block)
    }
}

/// The sprite of a human.
pub trait Sprite {
    /// Semi-tile for the head.
    fn head(&self) -> Foreground;

    /// Semi-tile for the up pointer.
    fn up(&self) -> Foreground;

    /// Semi-tile for the down pointer.
    fn down(&self) -> Foreground;

    /// Semi-tile for the left pointer.
    fn left(&self) -> Foreground;

    /// Semi-tile for the right pointer.
    fn right(&self) -> Foreground;
}
