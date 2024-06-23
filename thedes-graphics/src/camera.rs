use num::traits::{SaturatingAdd, SaturatingSub};
use thedes_domain::{
    game::Game,
    geometry::{Coord, CoordPair, Rect},
    map,
};
use thedes_tui::{
    color::{BasicColor, ColorPair},
    grapheme::NotGrapheme,
    tile::Tile,
    CanvasError,
    Tick,
};
use thiserror::Error;

use crate::{
    background::EntityTile as _,
    foreground::{EntityTile as _, PlayerHead, PlayerPointer},
};

#[derive(Debug, Error)]
#[error("Border maximum must be positive, found {given}")]
pub struct InvalidBorderMax {
    pub given: Coord,
}

#[derive(Debug, Error)]
#[error("Freedom minimum must be positive, found {given}")]
pub struct InvalidFreedomMin {
    pub given: Coord,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to manipulate screen canvas")]
    Canvas(
        #[from]
        #[source]
        CanvasError,
    ),
    #[error("Camera tried to access invalid map point")]
    InvalidMapPoint(
        #[from]
        #[source]
        map::InvalidPoint,
    ),
    #[error("Tried to intern invalid grapheme string")]
    NotGrapheme(
        #[from]
        #[source]
        NotGrapheme,
    ),
}

#[derive(Debug, Clone)]
pub struct Config {
    border_max: Coord,
    freedom_min: Coord,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self { border_max: 5, freedom_min: 1 }
    }

    pub fn with_border_max(
        self,
        border_max: Coord,
    ) -> Result<Self, InvalidBorderMax> {
        if border_max < 1 {
            Err(InvalidBorderMax { given: border_max })?
        }

        Ok(Self { border_max, ..self })
    }

    pub fn with_freedom_min(
        self,
        freedom_min: Coord,
    ) -> Result<Self, InvalidFreedomMin> {
        if freedom_min < 1 {
            Err(InvalidFreedomMin { given: freedom_min })?
        }

        Ok(Self { freedom_min, ..self })
    }

    pub fn finish(self) -> Camera {
        Camera::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    view: Rect,
    config: Config,
}

impl Camera {
    fn new(config: Config) -> Self {
        Self {
            view: Rect {
                top_left: CoordPair::from_axes(|_| 0),
                size: CoordPair::from_axes(|_| 0),
            },
            config,
        }
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
        game: &Game,
    ) -> Result<(), Error> {
        if !tick.will_render() {
            return Ok(());
        }

        self.update_camera(tick, game);

        tick.screen_mut().clear_canvas(BasicColor::Black.into())?;

        for y in self.view.top_left.y .. self.view.bottom_right().y {
            for x in self.view.top_left.x .. self.view.bottom_right().x {
                let point = CoordPair { y, x };
                let ground = game.map().get_ground(point)?;
                let color = ground.base_color();
                tick.screen_mut()
                    .mutate(point - self.view.top_left, |tile: Tile| Tile {
                        colors: ColorPair { background: color, ..tile.colors },
                        ..tile
                    })
                    .map_err(CanvasError::from)?;
            }
        }

        let player_head = PlayerHead;
        let player_head_color = player_head.base_color();
        let player_head_grapheme =
            player_head.grapheme(tick.screen_mut().grapheme_registry_mut())?;

        let player_pointer = PlayerPointer { facing: game.player().facing() };
        let player_pointer_color = player_pointer.base_color();
        let player_pointer_grapheme = player_pointer
            .grapheme(tick.screen_mut().grapheme_registry_mut())?;

        let foreground_tiles = [
            (game.player().head(), player_head_color, player_head_grapheme),
            (
                game.player().pointer(),
                player_pointer_color,
                player_pointer_grapheme,
            ),
        ];

        for (point, color, grapheme) in foreground_tiles {
            tick.screen_mut()
                .mutate(point - self.view.top_left, |tile: Tile| Tile {
                    colors: ColorPair { foreground: color, ..tile.colors },
                    grapheme,
                })
                .map_err(CanvasError::from)?;
        }

        Ok(())
    }

    fn border(&self) -> CoordPair {
        self.feasible_min_freedom().zip2_with(
            self.view.size,
            |min_freedom, size| {
                (size - min_freedom).min(self.config.border_max).max(1)
            },
        )
    }

    fn feasible_min_freedom(&self) -> CoordPair {
        self.view.size.map(|a| self.config.freedom_min.min(a.saturating_sub(1)))
    }

    fn freedom_view(&self) -> Rect {
        let border = self.border();
        Rect {
            top_left: self.view.top_left.saturating_add(&border),
            size: self.view.size.saturating_sub(&(border * 2)),
        }
    }

    fn update_camera(&mut self, tick: &Tick, game: &Game) {
        if self.view.size != tick.screen().canvas_size() {
            self.center_on_player(tick, game);
        } else if !self.freedom_view().contains_point(game.player().head()) {
            self.stick_to_border(game);
        } else if !self.view.contains_point(game.player().head())
            || !self.view.contains_point(game.player().pointer())
        {
            self.center_on_player(tick, game);
        }
    }

    fn center_on_player(&mut self, tick: &Tick, game: &Game) {
        let canvas_size = tick.screen().canvas_size();
        self.view = Rect {
            top_left: game.player().head().saturating_sub(&(canvas_size / 2)),
            size: canvas_size,
        };
    }

    fn stick_to_border(&mut self, game: &Game) {
        let border = self.border();
        let freedom_view = self.freedom_view();
        let head = game.player().head();
        let map_rect = game.map().rect();
        self.view.top_left = CoordPair::from_axes(|axis| {
            let start = if freedom_view.top_left[axis] > head[axis] {
                head[axis].saturating_sub(border[axis])
            } else if freedom_view.bottom_right()[axis] <= head[axis] {
                head[axis]
                    .saturating_sub(freedom_view.size[axis])
                    .saturating_sub(border[axis])
            } else {
                self.view.top_left[axis]
            };

            let start = if start < map_rect.top_left[axis] {
                map_rect.top_left[axis]
            } else if start >= map_rect.bottom_right()[axis] {
                start
                    .saturating_add(map_rect.size[axis])
                    .saturating_sub(self.view.size[axis])
            } else {
                start
            };

            start
        });
    }
}
