use thedes_domain::{
    game::Game,
    game2::{Game2, Game2Input, NewGameError},
    geometry::CoordPair,
};
use thedes_tui::{core::App, progress_bar};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to initialize game generator")]
    Init(
        #[source]
        #[from]
        thedes_gen::InitError,
    ),
    #[error("Failed to execute game generator")]
    Run(
        #[source]
        #[from]
        thedes_gen::Error,
    ),
    #[error("Failed to run progress bar")]
    Bar(
        #[source]
        #[from]
        progress_bar::Error,
    ),
    #[error("Failed to create new game")]
    NewGame(#[from] NewGameError),
}

#[derive(Debug, Clone)]
pub struct Component {
    bar: progress_bar::Component,
}

impl Component {
    pub fn new() -> Self {
        Self { bar: progress_bar::Component::new("Creating Game...") }
    }

    pub async fn run(
        &self,
        app: &mut App,
        config: thedes_gen::Config,
    ) -> Result<Option<Game2>, Error> {
        /*
        let generator = config.finish()?;
        let monitor = generator.progress_monitor();

        let maybe_game = self
            .bar
            .run(app, monitor, async move { generator.execute().await })
            .await?
            .transpose()?;
        */
        Ok(Some(Game2::new(Game2Input {
            player_head_pos: CoordPair { y: 1000, x: 1000 },
        })?))
    }
}
