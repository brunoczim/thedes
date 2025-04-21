use num::rational::Ratio;
use thedes_domain::{
    game::Game,
    geometry::{CoordPair, Rect},
    map::Map,
    player::PlayerPosition,
};
use thedes_geometry::orientation::Direction;
use thedes_session::Session;
use thedes_tui::core::App;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {}

#[derive(Debug, Clone)]
pub struct Config {
    control_events_per_tick: Ratio<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self { control_events_per_tick: Ratio::new(1, 8) }
    }

    pub fn with_control_events_per_tick(
        self,
        events: impl Into<Ratio<u32>>,
    ) -> Self {
        Self { control_events_per_tick: events.into(), ..self }
    }

    pub fn finish(self, game: Game) -> Component {
        Component {
            session: Session::new(game),
            control_events_per_tick: self.control_events_per_tick,
            controls_left: Ratio::new(0, 1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Component {
    session: Session,
    control_events_per_tick: Ratio<u32>,
    controls_left: Ratio<u32>,
}

impl Component {
    pub async fn run(self, _app: &mut App) -> Result<(), Error> {
        Ok(())
    }
}
