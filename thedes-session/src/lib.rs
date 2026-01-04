use camera::Camera;
use num::rational::Ratio;
use rand::{SeedableRng, distr::Distribution, rngs::StdRng};
use thedes_domain::{
    event,
    game::{Game, MovePlayerError},
    stat::StatValue,
};
use thedes_gen::event::{self as gen_event, EventDistr};
use thedes_geometry::orientation::Direction;
use thedes_tui::{
    core::{
        App,
        color::{BasicColor, ColorPair},
        geometry::{Coord, CoordPair},
    },
    text,
};

use thiserror::Error;

use crate::camera::DynamicStyle;

pub mod camera;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to handle session camera")]
    Camera(
        #[from]
        #[source]
        camera::Error,
    ),
    #[error("Failed to write HP hearts")]
    HpHearts(#[source] text::Error),
    #[error("Failed to write HP text")]
    HpText(#[source] text::Error),
}

#[derive(Debug, Error)]
pub enum EventError {
    #[error("Failed to create event distribution")]
    Distr(
        #[from]
        #[source]
        gen_event::DistrError,
    ),
    #[error("Failed to apply event to game")]
    Apply(
        #[from]
        #[source]
        event::ApplyError,
    ),
}

#[derive(Debug, Error)]
pub enum MoveAroundError {
    #[error("Failed to move player pointer")]
    MovePlayer(
        #[from]
        #[source]
        MovePlayerError,
    ),
}

#[derive(Debug, Error)]
pub enum QuickStepError {
    #[error("Failed to move player head")]
    MovePlayer(
        #[from]
        #[source]
        MovePlayerError,
    ),
}

#[derive(Debug, Clone)]
pub struct Config {
    camera: camera::Config,
    event_interval: Ratio<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            camera: camera::Config::new(),
            event_interval: Ratio::new(4, 100),
        }
    }

    pub fn with_camera(self, config: camera::Config) -> Self {
        Self { camera: config, ..self }
    }

    pub fn with_event_interval(self, ticks: Ratio<u64>) -> Self {
        Self { event_interval: ticks, ..self }
    }

    pub fn finish(self, game: Game) -> Session {
        Session {
            rng: StdRng::from_os_rng(),
            game,
            camera: self.camera.finish(),
            event_interval: self.event_interval,
            event_ticks: Ratio::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    rng: StdRng,
    game: Game,
    camera: Camera,
    event_interval: Ratio<u64>,
    event_ticks: Ratio<u64>,
}

impl Session {
    const STAT_VALUE_WIDTH: Coord = 3;
    const GAME_INFO_WIDTH: Coord = Self::STAT_VALUE_WIDTH * 2 + 1;

    const POS_HEIGHT: Coord = 1;

    pub fn render(&mut self, app: &mut App) -> Result<(), RenderError> {
        self.camera.render(
            app,
            &mut self.game,
            &DynamicStyle {
                margin_top_left: CoordPair { y: 1, x: Self::GAME_INFO_WIDTH },
                margin_bottom_right: CoordPair { y: 0, x: 0 },
            },
        )?;
        self.render_hp(app)?;
        Ok(())
    }

    pub fn tick_event(&mut self) -> Result<(), EventError> {
        self.event_ticks += 1;
        while self.event_ticks >= self.event_interval {
            self.event_ticks -= self.event_interval;
            let event = EventDistr::new(&self.game)?.sample(&mut self.rng);
            event.apply(&mut self.game)?;
        }
        Ok(())
    }

    pub fn move_around(
        &mut self,
        direction: Direction,
    ) -> Result<(), MoveAroundError> {
        self.game.move_player_pointer(direction)?;
        Ok(())
    }

    pub fn quick_step(
        &mut self,
        direction: Direction,
    ) -> Result<(), QuickStepError> {
        self.game.move_player_head(direction)?;
        Ok(())
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }

    fn render_hp(&self, app: &mut App) -> Result<(), RenderError> {
        let player_hp = self.game.player().hp();
        let width = StatValue::from(Self::GAME_INFO_WIDTH);
        let compensated_hearts =
            player_hp.value() * width + player_hp.curr_max() - 1;
        let heart_count = compensated_hearts / player_hp.curr_max();
        let heart_count = heart_count as usize;
        let hearts = "❤︎".repeat(heart_count);
        let hearts_point = CoordPair { y: Self::POS_HEIGHT, x: 0 };
        let hearts_colors = ColorPair {
            background: BasicColor::Black.into(),
            foreground: BasicColor::LightRed.into(),
        };
        text::inline(app, hearts_point, &hearts, hearts_colors)
            .map_err(RenderError::HpHearts)?;

        let numbers = format!(
            "{:>w$}/{:<w$}",
            player_hp.value(),
            player_hp.curr_max(),
            w = usize::from(Self::STAT_VALUE_WIDTH),
        );
        let hp_point = CoordPair { y: Self::POS_HEIGHT + 1, x: 0 };
        let hp_colors = ColorPair {
            background: BasicColor::Black.into(),
            foreground: BasicColor::White.into(),
        };
        text::inline(app, hp_point, &numbers, hp_colors)
            .map_err(RenderError::HpText)?;

        Ok(())
    }
}
