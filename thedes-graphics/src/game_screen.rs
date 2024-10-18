use std::{fmt, iter};

use thedes_domain::{
    game::Game,
    item::{self, Inventory, SlotEntry, StackableItem8},
};
use thedes_geometry::CoordPair;
use thedes_tui::{
    color::{BasicColor, Color, ColorPair, LegacyRgb, SetFg},
    geometry::Coord,
    grapheme::{self, NotGrapheme},
    tile::{MutateColors, MutationExt as _, SetGrapheme, Tile},
    CanvasError,
    InvalidCanvasPoint,
    TextStyle,
    Tick,
};
use thiserror::Error;

use crate::{
    camera::{self, Camera},
    tile::{foreground::Stick, Foreground},
    time::{self, circadian_cycle_icon},
    SessionData,
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
    #[error("Failed to get tiles for time info")]
    Time(
        #[from]
        #[source]
        time::Error,
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
    // 3 or 100%
    const INVENTORY_INFO_WIDTH: Coord = 4;
    // y or x
    const INVENTORY_ICON_WIDTH: Coord = 1;
    // |   3|y|
    const INVENTORY_WIDTH: Coord =
        1 + Self::INVENTORY_INFO_WIDTH + 1 + Self::INVENTORY_ICON_WIDTH + 1;
    const INVENTORY_HEIGHT: Coord =
        1 + 1 + 2 * (Inventory::SLOT_COUNT as Coord);

    const POS_HEIGHT: Coord = 1;

    // DAY:
    // 001 <sun/moon>
    // SEASON:
    // ware|summer|harvest|winter
    const GAME_INFO_WIDTH: Coord = 7;
    const GAME_INFO_Y_OFFSET: Coord = Self::POS_HEIGHT + 1;
    const GAME_DAY_NUMBER_WIDTH: Coord = 3;

    fn unselected_color() -> Color {
        LegacyRgb::new(4, 4, 4).into()
    }

    fn selected_color() -> Color {
        LegacyRgb::new(5, 5, 5).into()
    }

    fn new(config: Config) -> Self {
        Self { camera: config.camera.finish() }
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
        game: &Game,
        session_data: &SessionData,
    ) -> Result<(), Error> {
        tick.screen_mut().clear_canvas(BasicColor::Black.into())?;

        let camera_dynamic_style = camera::DynamicStyle {
            margin_top_left: CoordPair {
                y: Self::POS_HEIGHT,
                x: Self::GAME_INFO_WIDTH,
            },
            margin_bottom_right: CoordPair { y: 0, x: Self::INVENTORY_WIDTH },
        };
        self.camera.on_tick(tick, game, &camera_dynamic_style)?;

        let pos_string = format!("↱{}", game.player().position().head());
        tick.screen_mut().styled_text(&pos_string, &TextStyle::default())?;

        self.render_inventory(tick, game, session_data)?;

        self.render_game_info(tick, game)?;

        Ok(())
    }

    fn render_game_info(
        &mut self,
        tick: &mut Tick,
        game: &Game,
    ) -> Result<(), Error> {
        tick.screen_mut().inline_text(
            CoordPair { y: Self::GAME_INFO_Y_OFFSET, x: 0 },
            "DAY:",
            ColorPair::default(),
        )?;
        let width = usize::from(Self::GAME_DAY_NUMBER_WIDTH);
        let day = format!("{:0width$}", game.time().day() + 1);
        tick.screen_mut().inline_text(
            CoordPair { y: Self::GAME_INFO_Y_OFFSET + 1, x: 0 },
            &day,
            ColorPair::default(),
        )?;
        let circadian_cycle_tiles = circadian_cycle_icon(
            game.time(),
            tick.screen_mut().grapheme_registry_mut(),
        )?;
        for (i, tile) in circadian_cycle_tiles.into_iter().enumerate() {
            tick.screen_mut().set(
                CoordPair {
                    y: Self::GAME_INFO_Y_OFFSET + 1,
                    x: Self::GAME_DAY_NUMBER_WIDTH + 1 + (i as u16),
                },
                tile,
            )?;
        }
        tick.screen_mut().inline_text(
            CoordPair { y: Self::GAME_INFO_Y_OFFSET + 2, x: 0 },
            "SEASON:",
            ColorPair::default(),
        )?;
        tick.screen_mut().inline_text(
            CoordPair { y: Self::GAME_INFO_Y_OFFSET + 3, x: 0 },
            game.time().season().into_str(),
            ColorPair::default(),
        )?;
        Ok(())
    }

    fn render_inventory(
        &mut self,
        tick: &mut Tick,
        game: &Game,
        session_data: &SessionData,
    ) -> Result<(), Error> {
        self.render_inventory_grids(tick, session_data)?;

        for i in 0 .. Inventory::SLOT_COUNT {
            match game.player().inventory().get(i)? {
                SlotEntry::Vaccant => (),
                SlotEntry::Stackable8(entry) => {
                    let (base_color, grapheme) = match entry.item() {
                        StackableItem8::Stick => (
                            Stick.base_color(),
                            Stick.grapheme(
                                tick.screen_mut().grapheme_registry_mut(),
                            )?,
                        ),
                    };
                    self.render_inventory_icon(tick, i, base_color, grapheme)?;
                    self.render_inventory_info(
                        tick,
                        session_data,
                        i,
                        entry.count(),
                    )?;
                },
            }
        }

        Ok(())
    }

    fn render_inventory_icon(
        &mut self,
        tick: &mut Tick,
        index: usize,
        color: Color,
        grapheme: grapheme::Id,
    ) -> Result<(), Error> {
        let point = CoordPair {
            y: 1 + 2 * index as Coord,
            x: tick.screen().canvas_size().x - 1 - Self::INVENTORY_ICON_WIDTH,
        };
        let mutation = MutateColors(SetFg(color)).then(SetGrapheme(grapheme));
        tick.screen_mut().mutate(point, mutation)?;
        Ok(())
    }

    fn render_inventory_info(
        &mut self,
        tick: &mut Tick,
        session_data: &SessionData,
        index: usize,
        info: impl fmt::Display,
    ) -> Result<(), Error> {
        let point = CoordPair {
            y: 1 + 2 * index as Coord,
            x: tick.screen().canvas_size().x + 1 - Self::INVENTORY_WIDTH,
        };
        let text = format!(
            "{info:width$}",
            width = Self::INVENTORY_INFO_WIDTH as usize
        );
        let color = if index == session_data.selected_inventory_slot {
            Self::selected_color()
        } else {
            Self::unselected_color()
        };
        tick.screen_mut().inline_text(
            point,
            &text,
            ColorPair {
                foreground: color,
                background: BasicColor::Black.into(),
            },
        )?;
        Ok(())
    }

    fn render_inventory_grids(
        &mut self,
        tick: &mut Tick,
        session_data: &SessionData,
    ) -> Result<(), Error> {
        let bold_vert_pipe =
            tick.screen_mut().grapheme_registry_mut().get_or_register("┃")?;
        let bold_bottom_left =
            tick.screen_mut().grapheme_registry_mut().get_or_register("┗")?;
        let bold_horz_pipe =
            tick.screen_mut().grapheme_registry_mut().get_or_register("━")?;
        let horz_pipe =
            tick.screen_mut().grapheme_registry_mut().get_or_register("─")?;
        let double_horz_pipe =
            tick.screen_mut().grapheme_registry_mut().get_or_register("═")?;
        let vert_pipe =
            tick.screen_mut().grapheme_registry_mut().get_or_register("│")?;
        let double_vert_pipe =
            tick.screen_mut().grapheme_registry_mut().get_or_register("║")?;
        let ceil =
            tick.screen_mut().grapheme_registry_mut().get_or_register("┬")?;
        let floor =
            tick.screen_mut().grapheme_registry_mut().get_or_register("┴")?;
        let cross =
            tick.screen_mut().grapheme_registry_mut().get_or_register("┼")?;

        let top_left = CoordPair {
            y: 0,
            x: tick.screen().canvas_size().x - Self::INVENTORY_WIDTH,
        };

        let bold_vertical_pipe_len = Self::INVENTORY_HEIGHT - 1;
        for dy in 0 .. bold_vertical_pipe_len {
            let point = top_left + CoordPair { y: dy, x: 0 };
            tick.screen_mut().mutate(
                point,
                Tile {
                    colors: ColorPair {
                        foreground: Self::unselected_color(),
                        background: BasicColor::Black.into(),
                    },
                    grapheme: bold_vert_pipe,
                },
            )?;
        }

        let bottom_left =
            top_left + CoordPair { y: Self::INVENTORY_HEIGHT - 1, x: 0 };
        tick.screen_mut().mutate(
            bottom_left,
            Tile {
                colors: ColorPair {
                    foreground: Self::unselected_color(),
                    background: BasicColor::Black.into(),
                },
                grapheme: bold_bottom_left,
            },
        )?;

        for dx in 1 .. Self::INVENTORY_WIDTH {
            let point = bottom_left + CoordPair { y: 0, x: dx };
            tick.screen_mut().mutate(
                point,
                Tile {
                    colors: ColorPair {
                        foreground: Self::unselected_color(),
                        background: BasicColor::Black.into(),
                    },
                    grapheme: bold_horz_pipe,
                },
            )?;
        }

        for i in 0 ..= Inventory::SLOT_COUNT as Coord {
            let (color, grapheme) = if i
                == session_data.selected_inventory_slot as Coord
                || i.checked_sub(1).is_some_and(|prev_i| {
                    prev_i == session_data.selected_inventory_slot as Coord
                }) {
                (Self::selected_color(), double_horz_pipe)
            } else {
                (Self::unselected_color(), horz_pipe)
            };
            let dy = 2 * i;
            for dx in (1 ..= Self::INVENTORY_INFO_WIDTH)
                .chain(iter::once(Self::INVENTORY_INFO_WIDTH + 2))
            {
                let point = top_left + CoordPair { y: dy, x: dx };
                tick.screen_mut().mutate(
                    point,
                    Tile {
                        colors: ColorPair {
                            foreground: color,
                            background: BasicColor::Black.into(),
                        },
                        grapheme,
                    },
                )?;
            }
        }

        let vert_sep_dx =
            [Self::INVENTORY_INFO_WIDTH + 1, Self::INVENTORY_INFO_WIDTH + 3];

        for dx in vert_sep_dx {
            let point = top_left + CoordPair { y: 0, x: dx };
            tick.screen_mut().mutate(
                point,
                Tile {
                    colors: ColorPair {
                        foreground: Self::unselected_color(),
                        background: BasicColor::Black.into(),
                    },
                    grapheme: ceil,
                },
            )?;

            let point =
                top_left + CoordPair { y: Self::INVENTORY_HEIGHT - 2, x: dx };
            tick.screen_mut().mutate(
                point,
                Tile {
                    colors: ColorPair {
                        foreground: Self::unselected_color(),
                        background: BasicColor::Black.into(),
                    },
                    grapheme: floor,
                },
            )?;
        }

        for i in 0 .. Inventory::SLOT_COUNT as Coord {
            let (color, grapheme) =
                if i == session_data.selected_inventory_slot as Coord {
                    (Self::selected_color(), double_vert_pipe)
                } else {
                    (Self::unselected_color(), vert_pipe)
                };
            let dy = 2 * i + 1;
            for dx in vert_sep_dx {
                let point = top_left + CoordPair { y: dy, x: dx };
                tick.screen_mut().mutate(
                    point,
                    Tile {
                        colors: ColorPair {
                            foreground: color,
                            background: BasicColor::Black.into(),
                        },
                        grapheme,
                    },
                )?;
            }
        }

        for i in 1 .. Inventory::SLOT_COUNT as Coord {
            let dy = 2 * i;
            for dx in vert_sep_dx {
                let point = top_left + CoordPair { y: dy, x: dx };
                tick.screen_mut().mutate(
                    point,
                    Tile {
                        colors: ColorPair {
                            foreground: Self::unselected_color(),
                            background: BasicColor::Black.into(),
                        },
                        grapheme: cross,
                    },
                )?;
            }
        }

        Ok(())
    }
}
