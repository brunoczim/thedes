use crate::{
    entity,
    error::Result,
    graphics::{Color, Foreground, GString, Grapheme},
    math::plane::{Camera, Coord2, Direc, Nat},
    storage::save::{self, SavedGame},
    terminal,
};
use std::collections::HashSet;
use tokio::task;

/// Kind of a block.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Block {
    /// Empty.
    Empty,
    /// Wall block.
    Wall,
    /// An entity's physical part.
    Entity(entity::Physical),
}

impl Block {
    /// Renders this block on the screen.
    pub async fn render<'guard>(
        &self,
        pos: Coord2<Nat>,
        camera: Camera,
        screen: &mut terminal::Screen<'guard>,
        game: &SavedGame,
        rendered_entities: &mut HashSet<entity::Physical>,
    ) -> Result<()> {
        if let Some(inside_pos) = camera.convert(pos) {
            let bg = screen.get(inside_pos).colors.bg;
            let grapheme = match self {
                Block::Empty => Grapheme::new_lossy(" "),
                Block::Wall => draw_wall(pos, game).await?,
                Block::Entity(physical) => {
                    if rendered_entities.insert(physical.clone()) {
                        physical.render(camera, screen, game).await?;
                    }
                    return Ok(());
                },
            };
            let fg = Foreground { grapheme, color: Color::White };
            screen.set(inside_pos, fg.make_tile(bg));
        }

        Ok(())
    }

    /// Interacts with the user.
    pub async fn interact(
        &self,
        message: &mut GString,
        game: &SavedGame,
    ) -> Result<()> {
        match self {
            Block::Entity(physical) => {
                physical.interact(message, game).await?;
            },
            _ => (),
        }
        Ok(())
    }
}

async fn draw_wall(pos: Coord2<Nat>, game: &SavedGame) -> Result<Grapheme> {
    let direcs = [Direc::Up, Direc::Down, Direc::Left, Direc::Right];
    let mut has_block = [false; 4];
    for (i, &direc) in direcs.iter().enumerate() {
        if let Some(point) = pos.move_by_direc(direc) {
            has_block[i] = game.blocks().get(point).await? == Block::Wall;
            tracing::debug!(?i);
        }
    }

    let ch = match has_block {
        [true, true, true, true] => "┼",
        [true, true, true, false] => "┤",
        [true, true, false, true] => "├",
        [true, true, false, false] => "│",
        [true, false, true, true] => "┴",
        [true, false, true, false] => "┘",
        [true, false, false, true] => "└",
        [true, false, false, false] => "╵",

        [false, true, true, true] => "┬",
        [false, true, true, false] => "┐",
        [false, true, false, true] => "┌",
        [false, true, false, false] => "╷",
        [false, false, true, true] => "─",
        [false, false, true, false] => "╴",
        [false, false, false, true] => "╶",
        [false, false, false, false] => "∙",
    };
    Ok(Grapheme::new_lossy(ch))
}

/// A persitent map of blocks.
#[derive(Debug, Clone)]
pub struct Map {
    tree: sled::Tree,
}

impl Map {
    /// Creates a new map given a tree that stores blocks using coordinate pairs
    /// as keys. A seed is provided to create the noise function.
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = task::block_in_place(|| db.open_tree("block::Map"))?;
        Ok(Self { tree })
    }

    /// Sets a block at a given point.
    pub async fn set(&self, point: Coord2<Nat>, value: &Block) -> Result<()> {
        let point_vec = save::encode(point)?;
        let block_vec = save::encode(value)?;
        task::block_in_place(|| self.tree.insert(point_vec, block_vec))?;
        Ok(())
    }

    /// Gets a block at a given point.
    pub async fn get(&self, point: Coord2<Nat>) -> Result<Block> {
        let point_vec = save::encode(point)?;
        let res = task::block_in_place(|| self.tree.get(point_vec));

        match res? {
            Some(bytes) => Ok(save::decode(&bytes)?),
            None => {
                let block = Block::Empty;
                self.set(point, &block).await?;
                Ok(block)
            },
        }
    }
}
