use thedes_domain::{
    game::Game,
    item::{self, Inventory, SlotEntry, StackableItem8},
};
use thedes_geometry::CoordPair;
use thedes_tui::{
    color::{BasicColor, ColorPair},
    geometry::Coord,
    grapheme::NotGrapheme,
    tile::Tile,
    CanvasError,
    InvalidCanvasPoint,
    TextStyle,
    Tick,
};
use thiserror::Error;

use crate::{
    camera::{self, Camera},
    tile::{foreground::Stick, Foreground},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error manipulating camera")]
    Camera(
        #[from]
        #[source]
        camera::Error,
    ),
    #[error("Error rendering info")]
    Canvas(
        #[from]
        #[source]
        CanvasError,
    ),
    #[error("Programming error defining graphemes")]
    NotGrapheme(
        #[from]
        #[source]
        NotGrapheme,
    ),
    #[error("Manually accessing canvas point failed")]
    InvalidCanvasPoint(
        #[from]
        #[source]
        InvalidCanvasPoint,
    ),
    #[error("Failed to access inventory")]
    InventoryAccess(
        #[from]
        #[source]
        item::AccessError,
    ),
}

#[derive(Debug, Clone)]
pub struct Config {
    camera: camera::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self { camera: camera::Config::new() }
    }

    pub fn with_camera(self, camera_config: camera::Config) -> Self {
        Self { camera: camera_config, ..self }
    }

    pub fn finish(self) -> GameScreen {
        GameScreen::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct GameScreen {
    camera: Camera,
}

impl GameScreen {
    const INVENTORY_WIDTH: Coord = 4;
    const POS_HEIGHT: Coord = 1;

    fn new(config: Config) -> Self {
        Self { camera: config.camera.finish() }
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
        game: &Game,
    ) -> Result<(), Error> {
        tick.screen_mut().clear_canvas(BasicColor::Black.into())?;

        let camera_dynamic_style = camera::DynamicStyle {
            margin_top_left: CoordPair { y: Self::POS_HEIGHT, x: 0 },
            margin_bottom_right: CoordPair { y: 0, x: Self::INVENTORY_WIDTH },
        };
        self.camera.on_tick(tick, game, &camera_dynamic_style)?;

        let pos_string = format!("↱{}", game.player().position().head());
        tick.screen_mut().styled_text(&pos_string, &TextStyle::default())?;

        self.render_inventory(tick, game)?;

        Ok(())
    }

    fn render_inventory(
        &mut self,
        tick: &mut Tick,
        game: &Game,
    ) -> Result<(), Error> {
        let space =
            tick.screen_mut().grapheme_registry_mut().get_or_register(" ")?;
        let top_left_gr =
            tick.screen_mut().grapheme_registry_mut().get_or_register("╔")?;
        let top_right_gr =
            tick.screen_mut().grapheme_registry_mut().get_or_register("╕")?;
        let bot_left_gr =
            tick.screen_mut().grapheme_registry_mut().get_or_register("╚")?;
        let bot_right_gr =
            tick.screen_mut().grapheme_registry_mut().get_or_register("╛")?;
        let left_glue_gr =
            tick.screen_mut().grapheme_registry_mut().get_or_register("╠")?;
        let right_glue_gr =
            tick.screen_mut().grapheme_registry_mut().get_or_register("╡")?;
        let left_vert_glue_gr =
            tick.screen_mut().grapheme_registry_mut().get_or_register("║")?;
        let right_vert_glue_gr =
            tick.screen_mut().grapheme_registry_mut().get_or_register("│")?;
        let horz_glue_gr =
            tick.screen_mut().grapheme_registry_mut().get_or_register("═")?;

        let top_left = CoordPair {
            y: 0,
            x: tick.screen().canvas_size().x - Self::INVENTORY_WIDTH,
        };

        tick.screen_mut().mutate(
            top_left,
            Tile {
                colors: ColorPair {
                    background: BasicColor::Black.into(),
                    foreground: BasicColor::White.into(),
                },
                grapheme: top_left_gr,
            },
        )?;

        tick.screen_mut().mutate(
            top_left + CoordPair { y: 0, x: 2 },
            Tile {
                colors: ColorPair {
                    background: BasicColor::Black.into(),
                    foreground: BasicColor::White.into(),
                },
                grapheme: top_right_gr,
            },
        )?;

        tick.screen_mut().mutate(
            top_left
                + CoordPair { y: Inventory::SLOT_COUNT as Coord * 2, x: 0 },
            Tile {
                colors: ColorPair {
                    background: BasicColor::Black.into(),
                    foreground: BasicColor::White.into(),
                },
                grapheme: bot_left_gr,
            },
        )?;

        tick.screen_mut().mutate(
            top_left
                + CoordPair { y: Inventory::SLOT_COUNT as Coord * 2, x: 2 },
            Tile {
                colors: ColorPair {
                    background: BasicColor::Black.into(),
                    foreground: BasicColor::White.into(),
                },
                grapheme: bot_right_gr,
            },
        )?;

        for i in 0 ..= Inventory::SLOT_COUNT as Coord {
            tick.screen_mut().mutate(
                top_left + CoordPair { y: i * 2, x: 1 },
                Tile {
                    colors: ColorPair {
                        background: BasicColor::Black.into(),
                        foreground: BasicColor::White.into(),
                    },
                    grapheme: horz_glue_gr,
                },
            )?;
        }

        for i in 0 .. Inventory::SLOT_COUNT as Coord {
            tick.screen_mut().mutate(
                top_left + CoordPair { y: i * 2 + 1, x: 0 },
                Tile {
                    colors: ColorPair {
                        background: BasicColor::Black.into(),
                        foreground: BasicColor::White.into(),
                    },
                    grapheme: left_vert_glue_gr,
                },
            )?;
            tick.screen_mut().mutate(
                top_left + CoordPair { y: i * 2 + 1, x: 2 },
                Tile {
                    colors: ColorPair {
                        background: BasicColor::Black.into(),
                        foreground: BasicColor::White.into(),
                    },
                    grapheme: right_vert_glue_gr,
                },
            )?;

            let (base_color, grapheme) =
                match game.player().inventory().get(usize::from(i))? {
                    SlotEntry::Vaccant => (BasicColor::Black.into(), space),
                    SlotEntry::Stackable8(entry) => match entry.item() {
                        StackableItem8::Stick => (
                            Stick.base_color(),
                            Stick.grapheme(
                                tick.screen_mut().grapheme_registry_mut(),
                            )?,
                        ),
                    },
                };

            tick.screen_mut().mutate(
                top_left + CoordPair { y: 2 * i + 1, x: 1 },
                Tile {
                    colors: ColorPair {
                        background: BasicColor::Black.into(),
                        foreground: base_color,
                    },
                    grapheme,
                },
            )?;
        }

        for i in 1 .. Inventory::SLOT_COUNT as Coord {
            tick.screen_mut().mutate(
                top_left + CoordPair { y: i * 2, x: 0 },
                Tile {
                    colors: ColorPair {
                        background: BasicColor::Black.into(),
                        foreground: BasicColor::White.into(),
                    },
                    grapheme: left_glue_gr,
                },
            )?;
            tick.screen_mut().mutate(
                top_left + CoordPair { y: i * 2, x: 2 },
                Tile {
                    colors: ColorPair {
                        background: BasicColor::Black.into(),
                        foreground: BasicColor::White.into(),
                    },
                    grapheme: right_glue_gr,
                },
            )?;
        }

        Ok(())
    }
}
