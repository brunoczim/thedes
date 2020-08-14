use crate::{
    entity,
    error::Result,
    graphics::{BasicColor, CMYColor, Foreground, GString, Grapheme},
    math::{
        plane::{Camera, Coord2, Direc, Nat},
        rand::{weighted, Seed},
    },
    storage::save::SavedGame,
    terminal,
};
use rand::{rngs::StdRng, Rng};
use std::collections::HashSet;

const SEED_SALT: u64 = 0x10253E6093C603D;

type Weight = u64;

const WEIGHTS: &[weighted::Entry<Block, Weight>] = &[
    weighted::Entry { data: Block::Empty, weight: 100 * 100 },
    weighted::Entry { data: Block::Twig, weight: 1 },
];

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
    ///Small twigs for tools.
    Twig,
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
            let (grapheme, color) = match self {
                Block::Empty => {
                    (Grapheme::new_lossy(" "), BasicColor::White.into())
                },
                Block::Wall => {
                    (draw_wall(pos, game).await?, BasicColor::White.into())
                },
                Block::Twig => {
                    (Grapheme::new_lossy("y"), CMYColor::new(4, 3, 0).into())
                },
                Block::Entity(physical) => {
                    if rendered_entities.insert(physical.clone()) {
                        physical.render(camera, screen, game).await?;
                    }
                    return Ok(());
                },
            };
            let fg = Foreground { grapheme, color };
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
            has_block[i] = game.map().block(point).await? == Block::Wall;
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

/// A weighted generator of blocks.
#[derive(Debug, Clone)]
pub struct Generator {
    seed: Seed,
    weights: weighted::Entries<Block, Weight>,
}

impl Generator {
    /// Creates a new generator.
    pub fn new(seed: Seed) -> Generator {
        let weights = weighted::Entries::new(WEIGHTS.iter().cloned());
        Self { seed, weights }
    }

    pub fn block_at(&self, point: Coord2<Nat>) -> Block {
        self.seed
            .make_rng::<_, StdRng>((SEED_SALT, point))
            .sample(&self.weights)
            .data
            .clone()
    }
}
