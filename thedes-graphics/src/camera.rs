use num::traits::{SaturatingAdd, SaturatingSub};
use thedes_domain::{Coord, CoordPair, Game, Rect};
use thedes_tui::{RenderError, Tick};
use thiserror::Error;

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
    ) -> Result<(), RenderError> {
        if !tick.will_render() {
            return Ok(());
        }

        self.update_camera(tick, game);

        Ok(())
    }

    fn freedom_view(&self) -> Rect {
        Rect {
            top_left: self
                .view
                .top_left
                .saturating_sub_except(&self.config.freedom_min),
            ..self.view
        }
    }

    fn border(&self) -> CoordPair {
        self.view
            .size
            .saturating_sub_except(&self.config.border_max)
            .saturating_sub_except(&self.config.freedom_min)
            .map(|a| a.max(1))
    }

    fn update_camera(&mut self, tick: &Tick, game: &Game) {
        if !self.view.contains_point(game.player().head())
            || !self.view.contains_point(game.player().pointer())
            || self.view.size != game.map().rect().size
        {
            self.center_on_player(tick, game);
        }
        if !self.freedom_view().contains_point(game.player().head())
            || !self.freedom_view().contains_point(game.player().pointer())
        {
            self.stick_to_border(game);
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
        self.view.top_left = self.view.top_left.zip2_with(
            game.player().head().saturating_sub(&border),
            |coord, stick| coord.min(stick),
        );
        self.view.size = self.view.bottom_right().zip2_with(
            game.player()
                .head()
                .saturating_sub(&self.view.top_left)
                .saturating_add(&border),
            |coord, stick| coord.max(stick),
        );
    }
}
