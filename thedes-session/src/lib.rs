use camera::Camera;
use num::rational::Ratio;
use rand::{SeedableRng, distr::Distribution, rngs::StdRng};
use thedes_asset::Assets;
use thedes_dev::CommandContext;
use thedes_domain::{
    event::{self, MetaEvent},
    game::{Game, MovePlayerError},
    stat::StatValue,
};
use thedes_gen::event::{self as gen_event};
use thedes_geometry::orientation::Direction;
use thedes_settings::AudioSinkType;
use thedes_tui::{
    core::{
        App,
        audio::{self, PlayOptions, Volume},
        color::{BasicColor, ColorPair},
        geometry::{Coord, CoordPair},
        mutation::Set,
        screen,
    },
    text,
};

use thiserror::Error;

use crate::{camera::DynamicStyle, time::circadian_cycle_icon};

pub mod camera;

pub mod graphics;

mod time;

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
    #[error("Failed to write time information")]
    TimeInfo(#[source] text::Error),
    #[error("Failed to write season information key")]
    SeasonKey(#[source] text::Error),
    #[error("Failed to write season information ")]
    SeasonInfo(#[source] text::Error),
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
pub enum MetaEventError {
    #[error("Failed to flush audio commands")]
    FlushAudio(#[from] audio::FlushError),
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
    event_tick_size: u64,
    event_distr_config: gen_event::DistrConfig,
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
            event_tick_size: 2,
            event_distr_config: gen_event::DistrConfig::new(),
        }
    }

    pub fn with_camera(self, config: camera::Config) -> Self {
        Self { camera: config, ..self }
    }

    pub fn with_event_interval(self, ticks: Ratio<u64>) -> Self {
        Self { event_interval: ticks, ..self }
    }

    pub fn with_event_tick_size(self, size: u64) -> Self {
        Self { event_tick_size: size, ..self }
    }

    pub fn with_event_distr(self, config: gen_event::DistrConfig) -> Self {
        Self { event_distr_config: config, ..self }
    }

    pub fn finish(self, game: Game) -> Session {
        Session {
            rng: StdRng::from_os_rng(),
            game,
            camera: self.camera.finish(),
            event_interval: self.event_interval,
            event_ticks: Ratio::ZERO,
            event_tick_size: self.event_tick_size,
            event_distr_config: self.event_distr_config,
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
    event_tick_size: u64,
    event_distr_config: gen_event::DistrConfig,
}

impl Session {
    const STAT_VALUE_WIDTH: Coord = 3;
    const GAME_INFO_WIDTH: Coord = Self::STAT_VALUE_WIDTH * 2 + 1;

    const POS_HEIGHT: Coord = 1;
    const HP_HEIGHT: Coord = 2;

    const TIME_INFO_Y_OFFSET: Coord =
        Self::POS_HEIGHT + 1 + Self::HP_HEIGHT + 1;

    // DAY:
    // 001 <sun/moon>
    // SEASON:
    // ware|summer|harvest|winter
    const GAME_INFO_Y_OFFSET: Coord = Self::POS_HEIGHT + 1;
    const GAME_DAY_NUMBER_WIDTH: Coord = 3;

    pub fn render(&mut self, app: &mut App) -> Result<(), RenderError> {
        self.camera.render(
            app,
            &self.game,
            &DynamicStyle {
                margin_top_left: CoordPair { y: 1, x: Self::GAME_INFO_WIDTH },
                margin_bottom_right: CoordPair { y: 0, x: 0 },
            },
        )?;
        self.render_hp(app)?;
        self.render_time_info(app)?;
        Ok(())
    }

    pub fn tick_event(&mut self) -> Result<(), EventError> {
        self.game_mut().tick();
        self.event_ticks += self.event_tick_size;
        while self.event_ticks >= self.event_interval {
            self.event_ticks -= self.event_interval;
            let event = self
                .event_distr_config
                .finish(&self.game)?
                .sample(&mut self.rng);
            self.game.schedule_event(event, 0);
            self.game.execute_events()?;
        }
        Ok(())
    }

    pub fn consume_meta_events(
        &mut self,
        app: &mut App,
        assets: &'static Assets,
    ) -> Result<(), MetaEventError> {
        while let Some(meta_event) = self.game_mut().read_one_meta_event() {
            match meta_event {
                MetaEvent::MonsterHit(pos) => {
                    self.consume_monster_hit(pos, app, assets)
                },
                MetaEvent::MonsterGrowl(pos) => {
                    self.consume_monster_growl(pos, app, assets)
                },
            }
        }

        app.audio_controller.flush()?;

        Ok(())
    }

    fn consume_monster_hit(
        &self,
        pos: CoordPair,
        app: &mut App,
        assets: &'static Assets,
    ) {
        if !self.game().player().position().contains(pos) {
            return;
        }
        app.audio_controller.queue([audio::Command::new_play_once(
            AudioSinkType::Fx,
            &assets.sound.hit[..],
        )]);
    }

    fn consume_monster_growl(
        &self,
        pos: CoordPair,
        app: &mut App,
        assets: &'static Assets,
    ) {
        if !self.camera.contains(pos) {
            return;
        }

        let distances = self
            .game()
            .player()
            .position()
            .head()
            .zip2_with(pos, Coord::abs_diff);
        let distance = (distances.y + distances.x) as u64;
        let distance_total = self.camera.half_view_perimeter() as u64;
        let volume_max = Volume::MAX as u64;
        let relative_volume =
            (distance * volume_max / distance_total) as Volume;

        app.audio_controller.queue([audio::Command::new_play_once_with(
            AudioSinkType::Fx,
            &assets.sound.growl[..],
            PlayOptions { relative_volume },
        )]);
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

    pub fn dev_command_context<'a>(&'a mut self) -> CommandContext<'a, 'a> {
        CommandContext {
            game: &mut self.game,
            event_distr_config: &mut self.event_distr_config,
        }
    }

    fn render_hp(&self, app: &mut App) -> Result<(), RenderError> {
        let player_hp = self.game.player().hp();
        let width = StatValue::from(Self::GAME_INFO_WIDTH);
        let compensated_hearts =
            player_hp.value() * width + player_hp.curr_max() - 1;
        let heart_count = compensated_hearts / player_hp.curr_max();
        let heart_count = heart_count as usize;
        let empty_heart_count = width as usize - heart_count;
        let hearts = "♥︎".repeat(heart_count) + &"♡".repeat(empty_heart_count);

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

    fn render_time_info(&self, app: &mut App) -> Result<(), RenderError> {
        text::inline(
            app,
            CoordPair { y: Self::TIME_INFO_Y_OFFSET, x: 0 },
            "DAY:",
            ColorPair::default(),
        )
        .map_err(RenderError::TimeInfo)?;
        let width = usize::from(Self::GAME_DAY_NUMBER_WIDTH);
        let day = format!("{:0width$}", self.game.time().day() + 1);
        text::inline(
            app,
            CoordPair { y: Self::GAME_INFO_Y_OFFSET + 1, x: 0 },
            &day,
            ColorPair::default(),
        )
        .map_err(RenderError::TimeInfo)?;

        let circadian_cycle_tiles =
            circadian_cycle_icon(self.game.time(), &mut app.grapheme_registry);
        for (i, tile) in circadian_cycle_tiles.into_iter().enumerate() {
            app.canvas.queue([screen::Command::new_mutation(
                CoordPair {
                    y: Self::GAME_INFO_Y_OFFSET + 1,
                    x: Self::GAME_DAY_NUMBER_WIDTH + 1 + (i as u16),
                },
                Set(tile),
            )]);
        }

        text::inline(
            app,
            CoordPair { y: Self::GAME_INFO_Y_OFFSET + 2, x: 0 },
            "SEASON:",
            ColorPair::default(),
        )
        .map_err(RenderError::SeasonKey)?;
        text::inline(
            app,
            CoordPair { y: Self::GAME_INFO_Y_OFFSET + 3, x: 0 },
            self.game.time().season().into_str(),
            ColorPair::default(),
        )
        .map_err(RenderError::SeasonInfo)?;

        Ok(())
    }
}
