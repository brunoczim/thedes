use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, BufReader},
};

use num::rational::Ratio;
use serde::{Deserialize, Serialize};
use thedes_domain::{
    game::Game,
    time::{CircadianCycleStep, LunarPhase, Season},
};
use thedes_tui::{
    color::BasicColor,
    event::{Event, Key, KeyEvent},
    TextStyle,
    Tick,
};
use thiserror::Error;

pub const PATH: &str = "thedes-cmd.json";

#[derive(Debug, Error)]
pub enum TickError {
    #[error(transparent)]
    Render(#[from] thedes_tui::CanvasError),
    #[error("Failed to run command(s)")]
    Run(
        #[from]
        #[source]
        RunError,
    ),
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("Failed reading {PATH}")]
    Read(
        #[from]
        #[source]
        io::Error,
    ),
    #[error("Failed decoding command")]
    Decode(
        #[from]
        #[source]
        serde_json::Error,
    ),
    #[error("Unknown key {:?}", .0)]
    UnknownKey(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Action {
    Run(char),
    RunPrevious,
    Exit,
}

#[derive(Debug, Clone)]
pub struct Component {
    previous: char,
}

impl Component {
    pub const DEFAULT_KEY: char = '.';

    pub fn new() -> Self {
        Self { previous: Self::DEFAULT_KEY }
    }

    pub fn reset(&mut self) {
        self.previous = Self::DEFAULT_KEY;
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
        game: &mut Game,
    ) -> Result<bool, TickError> {
        self.render(tick)?;
        match self.handle_events(tick) {
            Some(Action::Run(ch)) => {
                self.run(ch, game);
                Ok(false)
            },
            Some(Action::RunPrevious) => {
                self.run_previous(game);
                Ok(false)
            },
            Some(Action::Exit) => Ok(false),
            None => Ok(true),
        }
    }

    pub fn run_previous(&mut self, game: &mut Game) {
        self.run(self.previous, game);
    }

    fn run(&mut self, ch: char, game: &mut Game) {
        self.previous = ch;
        if let Err(error) = run(ch, game) {
            tracing::error!("Failed running development script: {}", error);
            tracing::warn!("Caused by:");
            let mut source = error.source();
            while let Some(current) = source {
                tracing::warn!("- {}", current);
                source = current.source();
            }
        }
    }

    fn render(&mut self, tick: &mut Tick) -> Result<(), TickError> {
        tick.screen_mut().clear_canvas(BasicColor::Black.into())?;
        tick.screen_mut().styled_text(
            "Debug/Test Script Mode",
            &TextStyle::default().with_top_margin(1).with_align(1, 2),
        )?;
        tick.screen_mut().styled_text(
            "Press any character key to run a corresponding script.",
            &TextStyle::default().with_top_margin(4).with_align(1, 2),
        )?;
        tick.screen_mut().styled_text(
            "Press enter to run previous script or the default one.",
            &TextStyle::default().with_top_margin(4).with_align(1, 2),
        )?;
        tick.screen_mut().styled_text(
            "Press ESC to cancel.",
            &TextStyle::default().with_top_margin(6).with_align(1, 2),
        )?;
        Ok(())
    }

    fn handle_events(&mut self, tick: &mut Tick) -> Option<Action> {
        while let Some(event) = tick.next_event() {
            match event {
                Event::Key(KeyEvent { main_key: Key::Char(ch), .. }) => {
                    return Some(Action::Run(ch))
                },

                Event::Key(KeyEvent { main_key: Key::Enter, .. }) => {
                    return Some(Action::RunPrevious)
                },

                Event::Key(KeyEvent { main_key: Key::Esc, .. }) => {
                    return Some(Action::Exit)
                },

                _ => (),
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct ScriptTable {
    scripts: HashMap<char, Script>,
}

impl ScriptTable {
    pub fn run(&self, key: char, game: &mut Game) -> Result<(), RunError> {
        match self.scripts.get(&key) {
            Some(script) => {
                script.run(game);
                Ok(())
            },
            None => Err(RunError::UnknownKey(key)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum Script {
    Single(CommandBlock),
    List(Vec<CommandBlock>),
}

impl Script {
    fn run(&self, game: &mut Game) {
        match self {
            Self::Single(block) => block.run(game),
            Self::List(list) => {
                for block in list {
                    block.run(game)
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandBlock {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_time: Option<SetTimeCommand>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_in_day_clock: Option<SetInDayClock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shift_in_day_clock: Option<ShiftInDayClock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_circadian_cycle_step: Option<SetCircadianCycleStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_day: Option<SetDay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    add_days: Option<AddDays>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_lunar_phase: Option<SetLunarPhase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_season: Option<SetSeason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_day_of_year: Option<SetDayOfYear>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_year: Option<SetYear>,
}

impl CommandBlock {
    fn run(&self, game: &mut Game) {
        if let Some(command) = &self.set_time {
            command.run(game);
        }
        if let Some(command) = &self.set_year {
            command.run(game);
        }
        if let Some(command) = &self.set_day {
            command.run(game);
        }
        if let Some(command) = &self.add_days {
            command.run(game);
        }
        if let Some(command) = &self.set_day_of_year {
            command.run(game);
        }
        if let Some(command) = &self.set_season {
            command.run(game);
        }
        if let Some(command) = &self.set_lunar_phase {
            command.run(game);
        }
        if let Some(command) = &self.set_in_day_clock {
            command.run(game);
        }
        if let Some(command) = &self.shift_in_day_clock {
            command.run(game);
        }
        if let Some(command) = &self.set_circadian_cycle_step {
            command.run(game);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetTimeCommand {
    stamp: u64,
}

impl SetTimeCommand {
    fn run(&self, game: &mut Game) {
        game.debug_time().set(self.stamp);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum SetInDayClock {
    Float(f64),
    Ratio(u64, u64),
}

impl SetInDayClock {
    const PRECISION: u64 = 1_000_000;

    fn run(&self, game: &mut Game) {
        game.debug_time().set_in_day_clock(match self {
            Self::Float(real) => Ratio::new(
                (real * Self::PRECISION as f64) as u64,
                Self::PRECISION,
            ),
            Self::Ratio(numer, denom) => Ratio::new(*numer, *denom),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum ShiftInDayClock {
    Float(f64),
    Ratio(i64, i64),
}

impl ShiftInDayClock {
    const PRECISION: i64 = 1_000_000;

    fn run(&self, game: &mut Game) {
        let ratio = match self {
            Self::Float(real) => Ratio::new(
                (real * Self::PRECISION as f64) as i64,
                Self::PRECISION,
            ),
            Self::Ratio(numer, denom) => Ratio::new(*numer, *denom),
        };
        let (curr_numer, curr_denom) = game.time().in_day_clock().into_raw();
        let curr = Ratio::new(
            i64::try_from(curr_numer).unwrap_or(i64::MAX),
            i64::try_from(curr_denom).unwrap_or(i64::MAX),
        );
        let shifted_in_day_clock = curr + (ratio % 1 + 1) % 1;
        let (shifted_numer, shifted_denom) = shifted_in_day_clock.into_raw();
        let new_in_day_clock =
            Ratio::new(shifted_numer as u64, shifted_denom as u64);
        game.debug_time().set_in_day_clock(new_in_day_clock);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetCircadianCycleStep {
    step: CircadianCycleStep,
}

impl SetCircadianCycleStep {
    fn run(&self, game: &mut Game) {
        game.debug_time().set_circadian_cycle_step(self.step);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetDay {
    day: u64,
}

impl SetDay {
    fn run(&self, game: &mut Game) {
        game.debug_time().set_day(self.day);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct AddDays {
    delta: i64,
}

impl AddDays {
    fn run(&self, game: &mut Game) {
        let day = game.time().day();
        let shifted_day = self.delta.saturating_add_unsigned(day);
        let new_day = u64::try_from(shifted_day).unwrap_or_default();
        game.debug_time().set_day(new_day);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetLunarPhase {
    phase: LunarPhase,
}

impl SetLunarPhase {
    fn run(&self, game: &mut Game) {
        game.debug_time().set_lunar_phase(self.phase);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetSeason {
    season: Season,
}

impl SetSeason {
    fn run(&self, game: &mut Game) {
        game.debug_time().set_season(self.season);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetDayOfYear {
    day: u64,
}

impl SetDayOfYear {
    fn run(&self, game: &mut Game) {
        game.debug_time().set_day_of_year(self.day);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetYear {
    year: u64,
}

impl SetYear {
    fn run(&self, game: &mut Game) {
        game.debug_time().set_year(self.year);
    }
}

fn read_table() -> Result<ScriptTable, RunError> {
    let file = File::open(PATH)?;
    let reader = BufReader::new(file);
    let script = serde_json::from_reader(reader)?;
    Ok(script)
}

pub fn run(key: char, game: &mut Game) -> Result<(), RunError> {
    let table = read_table()?;
    table.run(key, game)?;
    Ok(())
}
