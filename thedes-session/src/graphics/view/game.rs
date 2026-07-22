use thedes_domain::{
    block::{Block, PlaceableBlock, SpecialBlock},
    game::Game,
    geometry::Rect,
    map::AccessError,
    monster::InvalidId,
};
use thedes_geometry::CoordPair;
use thiserror::Error;

use crate::graphics::{
    tile::{
        Renderer as _,
        foreground::{MonsterHead, PlayerHead, PlayerPointer},
    },
    view::Renderer,
};

use super::Viewable;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error accessing map element")]
    Access(
        #[from]
        #[source]
        AccessError,
    ),
    #[error("Failed to find monster")]
    GetMonster(#[from] InvalidId),
    #[error("Error rendering map element")]
    RenderElement(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Viewable for Game {
    type Error = Error;

    fn render<R>(&self, rect: Rect, mut renderer: R) -> Result<(), Self::Error>
    where
        R: Renderer,
    {
        for y in rect.top_left.y .. rect.bottom_right().y {
            for x in rect.top_left.x .. rect.bottom_right().x {
                let point = CoordPair { y, x };
                let relative_point = CoordPair {
                    y: y - rect.top_left.y,
                    x: x - rect.top_left.x,
                };
                let mut sub_renderer = renderer.tile_renderer(relative_point);
                let ground = self.map().get_ground(point)?;
                sub_renderer
                    .render_background(ground)
                    .map_err(|e| Error::RenderElement(Box::new(e)))?;

                match self.map().get_block(point)? {
                    Block::Special(SpecialBlock::Player) => {
                        if self.player().position().head() == point {
                            sub_renderer
                                .render_foreground(PlayerHead)
                                .map_err(|e| {
                                    Error::RenderElement(Box::new(e))
                                })?;
                        } else {
                            let facing = self.player().position().facing();
                            sub_renderer
                                .render_foreground(PlayerPointer { facing })
                                .map_err(|e| {
                                    Error::RenderElement(Box::new(e))
                                })?;
                        }
                    },

                    Block::Placeable(PlaceableBlock::Air) => (),

                    Block::Special(SpecialBlock::Monster(id)) => {
                        let monster_pos =
                            self.monster_registry().get_by_id(id)?.position();
                        sub_renderer
                            .render_foreground(MonsterHead {
                                facing: monster_pos.facing(),
                            })
                            .map_err(|e| Error::RenderElement(Box::new(e)))?;
                    },
                }
            }
        }
        Ok(())
    }
}
