use crate::{
    entity,
    error::Result,
    map::Coord,
    math::rand::{weighted, Seed},
    session::Camera,
    storage::save::SavedGame,
};
use andiskaz::{
    color::{BasicColor, CmyColor},
    screen::Screen,
    string::{TermGrapheme, TermString},
    tile::Foreground,
};
use gardiz::{
    coord::Vec2,
    direc::{DirecMap, Direction},
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
        pos: Vec2<Coord>,
        camera: Camera,
        screen: &mut Screen<'guard>,
        game: &SavedGame,
        rendered_entities: &mut HashSet<entity::Physical>,
    ) -> Result<()> {
        if let Some(inside_pos) = camera.convert(pos) {
            let (grapheme, color) = match self {
                Block::Empty => {
                    (TermGrapheme::new_lossy(" "), BasicColor::White.into())
                },
                Block::Wall => {
                    (draw_wall(pos, game).await?, BasicColor::White.into())
                },
                Block::Twig => (
                    TermGrapheme::new_lossy("y"),
                    CmyColor::new(4, 3, 0).into(),
                ),
                Block::Entity(physical) => {
                    if rendered_entities.insert(physical.clone()) {
                        physical.render(camera, screen, game).await?;
                    }
                    return Ok(());
                },
            };
            screen.set(inside_pos, Foreground { grapheme, color });
        }

        Ok(())
    }

    /// Interacts with the user.
    pub async fn interact(
        &self,
        message: &mut TermString,
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

async fn draw_wall(pos: Vec2<Coord>, game: &SavedGame) -> Result<TermGrapheme> {
    let mut has_block = DirecMap::from_direcs(|_| false);
    for direction in Direction::iter() {
        if let Some(point) = pos.checked_move(direction) {
            let curr_block = game.map().block(point).await?;
            has_block[direction] = curr_block == Block::Wall;
        }
    }

    let ch = match has_block {
        DirecMap { up: true, down: true, left: true, right: true } => "┼",
        DirecMap { up: true, down: true, left: true, right: false } => "┤",
        DirecMap { up: true, down: true, left: false, right: true } => "├",
        DirecMap { up: true, down: true, left: false, right: false } => "│",
        DirecMap { up: true, down: false, left: true, right: true } => "┴",
        DirecMap { up: true, down: false, left: true, right: false } => "┘",
        DirecMap { up: true, down: false, left: false, right: true } => "└",
        DirecMap { up: true, down: false, left: false, right: false } => "╵",
        DirecMap { up: false, down: true, left: true, right: true } => "┬",
        DirecMap { up: false, down: true, left: true, right: false } => "┐",
        DirecMap { up: false, down: true, left: false, right: true } => "┌",
        DirecMap { up: false, down: true, left: false, right: false } => "╷",
        DirecMap { up: false, down: false, left: true, right: true } => "─",
        DirecMap { up: false, down: false, left: true, right: false } => "╴",
        DirecMap { up: false, down: false, left: false, right: true } => "╶",
        DirecMap { up: false, down: false, left: false, right: false } => "∙",
    };
    Ok(TermGrapheme::new_lossy(ch))
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

    pub fn block_at(&self, point: Vec2<Coord>) -> Block {
        self.seed
            .make_rng::<_, StdRng>((SEED_SALT, point))
            .sample(&self.weights)
            .data
            .clone()
    }
}
