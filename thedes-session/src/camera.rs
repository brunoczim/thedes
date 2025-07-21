use num::traits::{SaturatingAdd, SaturatingSub};
use thedes_domain::{
    block::{Block, PlaceableBlock, SpecialBlock},
    game::Game,
    geometry::{Coord, CoordPair, Rect},
    map,
    matter::Ground,
    monster,
};
use thedes_geometry::orientation::Direction;
use thedes_tui::{
    core::{
        App,
        color::{
            BasicColor,
            Rgb,
            mutation::{MutateBg, MutateFg},
        },
        grapheme,
        mutation::{MutationExt, Set},
        screen,
        tile::{MutateColors, MutateGrapheme},
    },
    text,
};
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

#[derive(Debug, Error)]
pub enum Error {
    #[error("Camera failed to access map data")]
    MapAccess(
        #[from]
        #[source]
        map::AccessError,
    ),
    #[error("Failed to render text")]
    Text(
        #[from]
        #[source]
        text::Error,
    ),
    #[error("Found invalid monster ID")]
    InvalidMonsterId(
        #[from]
        #[source]
        monster::InvalidId,
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

    pub(crate) fn finish(self) -> Camera {
        Camera::new(self)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Camera {
    view: Rect,
    offset: CoordPair,
    config: Config,
}

impl Camera {
    fn new(config: Config) -> Self {
        Self {
            view: Rect {
                top_left: CoordPair::from_axes(|_| 0),
                size: CoordPair::from_axes(|_| 0),
            },
            offset: CoordPair { y: 1, x: 0 },
            config,
        }
    }

    pub fn update(&mut self, app: &mut App, game: &Game) {
        if self.view.size != app.canvas.size() - self.offset {
            self.center_on_player(app, game);
        } else if !self
            .freedom_view()
            .contains_point(game.player().position().head())
        {
            self.stick_to_border(game);
        } else if !self.view.contains_point(game.player().position().head())
            || !self.view.contains_point(game.player().position().pointer())
        {
            self.center_on_player(app, game);
        }
    }

    pub fn render(&mut self, app: &mut App, game: &Game) -> Result<(), Error> {
        app.canvas
            .queue([screen::Command::new_clear_screen(BasicColor::Black)]);

        let pos_string = format!("↱{}", game.player().position().head());
        text::styled(app, &pos_string, &text::Style::default())?;

        for y in self.view.top_left.y .. self.view.bottom_right().y {
            for x in self.view.top_left.x .. self.view.bottom_right().x {
                let player_pos = game.player().position();
                let point = CoordPair { y, x };
                let canvas_point = point - self.view.top_left + self.offset;

                let ground = game.map().get_ground(point)?;
                let bg_color = match ground {
                    Ground::Grass => Rgb::new(0x00, 0xff, 0x80).into(),
                    Ground::Sand => Rgb::new(0xff, 0xff, 0x80).into(),
                    Ground::Stone => Rgb::new(0xc0, 0xc0, 0xc0).into(),
                };

                let block = game.map().get_block(point)?;

                let fg_color = BasicColor::Black.into();
                let char = match block {
                    Block::Special(SpecialBlock::Player) => {
                        if player_pos.head() == point {
                            'O'
                        } else {
                            match player_pos.facing() {
                                Direction::Up => 'Ʌ',
                                Direction::Down => 'V',
                                Direction::Left => '<',
                                Direction::Right => '>',
                            }
                        }
                    },
                    Block::Special(SpecialBlock::Monster(id)) => {
                        let monster_pos =
                            game.monster_registry().get_by_id(id)?.position();
                        match monster_pos.facing() {
                            Direction::Up => 'ɷ',
                            Direction::Down => 'ʊ',
                            Direction::Left => 'ɞ',
                            Direction::Right => 'ʚ',
                        }
                    },
                    Block::Placeable(PlaceableBlock::Air) => ' ',
                };
                let grapheme = grapheme::Id::from(char);

                let bg_mutation = MutateBg(Set(bg_color));
                let fg_mutation = MutateFg(Set(fg_color));
                let color_mutation =
                    MutateColors(bg_mutation.then(fg_mutation));
                let grapheme_mutation = MutateGrapheme(Set(grapheme));
                let mutation = color_mutation.then(grapheme_mutation);

                app.canvas.queue([screen::Command::new_mutation(
                    canvas_point,
                    mutation,
                )]);
            }
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
        self.view
            .size
            .map(|coord| self.config.freedom_min.min(coord.saturating_sub(1)))
    }

    fn freedom_view(&self) -> Rect {
        let border = self.border();
        Rect {
            top_left: self.view.top_left.saturating_add(&border),
            size: self.view.size.saturating_sub(&(border * 2)),
        }
    }

    fn center_on_player(&mut self, app: &mut App, game: &Game) {
        let view_size = app.canvas.size() - self.offset;
        self.view = Rect {
            top_left: game
                .player()
                .position()
                .head()
                .saturating_sub(&(view_size / 2)),
            size: view_size,
        };
    }

    fn stick_to_border(&mut self, game: &Game) {
        let border = self.border();
        let freedom_view = self.freedom_view();
        let head = game.player().position().head();
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

            start.max(map_rect.top_left[axis]).min(
                map_rect.bottom_right()[axis]
                    .saturating_sub(self.view.size[axis]),
            )
        });
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use thedes_domain::{game::Game, map::Map, player::PlayerPosition};
    use thedes_geometry::{CoordPair, Rect, orientation::Direction};
    use thedes_tui::core::{
        App,
        runtime::{self, device::mock::RuntimeDeviceMock},
        screen,
    };
    use tokio::task;

    struct SetupArgs {
        map_rect: thedes_domain::geometry::Rect,
        player_head: thedes_domain::geometry::CoordPair,
        player_facing: Direction,
        camera: super::Config,
    }

    struct Resources {
        game: Game,
        camera: super::Camera,
        device_mock: RuntimeDeviceMock,
        runtime_config: runtime::Config,
    }

    fn setup_resources(args: SetupArgs) -> Resources {
        let map = Map::new(args.map_rect).unwrap();
        let player_position =
            PlayerPosition::new(args.player_head, args.player_facing).unwrap();
        let game = Game::new(map, player_position).unwrap();
        let camera = args.camera.finish();

        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        let device = device_mock.open();
        let runtime_config = runtime::Config::new()
            .with_screen(
                screen::Config::new()
                    .with_canvas_size(CoordPair { y: 22, x: 78 }),
            )
            .with_device(device);

        Resources { game, camera, runtime_config, device_mock }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn correct_initial_view_min_commands() {
        let Resources { game, mut camera, device_mock, runtime_config } =
            setup_resources(SetupArgs {
                map_rect: Rect {
                    top_left: CoordPair { x: 500, y: 600 },
                    size: CoordPair { x: 1000, y: 1050 },
                },
                player_head: CoordPair { y: 710, x: 1203 },
                player_facing: Direction::Up,
                camera: super::Config::default(),
            });

        device_mock.screen().enable_command_log();

        let main = |mut app: App| async move {
            camera.update(&mut app, &game);
            camera.render(&mut app, &game).unwrap();
            app.canvas.flush().unwrap();
            app.tick_session.tick().await;
            app.tick_session.tick().await;
        };

        let runtime_future = task::spawn(runtime_config.run(main));
        tokio::time::sleep(Duration::from_millis(50)).await;
        runtime_future.await.unwrap().unwrap();

        let command_log =
            device_mock.screen().take_command_log().unwrap().concat();

        let expected_min_len = (22 - 1) * 78 + 1;
        assert!(
            command_log.len() >= expected_min_len,
            "left: {}\nright: {}",
            command_log.len(),
            expected_min_len,
        );
    }
}
