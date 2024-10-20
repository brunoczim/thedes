use std::{
    fmt,
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use num::rational::Ratio;
use serde::{Deserialize, Serialize};
use thedes_domain::{
    game::Game,
    time::{CircadianCycleStep, LunarPhase, Season},
};
use thedes_tui::{
    component::{
        menu::{self, Menu},
        Cancellable,
        CancellableOutput,
    },
    Tick,
};
use thiserror::Error;

pub const DEFAULT_PATH: &str = "thedes-cmd.json";

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
#[error("Inconsistent lookup of menu option")]
pub struct ResetError {
    #[from]
    source: menu::UnknownOption<MenuOption>,
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("Failed reading {DEFAULT_PATH}")]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MenuOption {
    Suffixed(u8),
    Plain,
}

impl fmt::Display for MenuOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Suffixed(i) => write!(f, "thedes-cmd_{i}.json"),
            Self::Plain => write!(f, "thedes-cmd.json"),
        }
    }
}

impl menu::OptionItem for MenuOption {}

#[derive(Debug, Clone)]
pub struct Component {
    menu: Menu<MenuOption, Cancellable>,
}

impl Component {
    pub fn new() -> Self {
        let mut options = menu::Options::with_initial(MenuOption::Suffixed(1));
        for i in 2 ..= 9 {
            options = options.add(MenuOption::Suffixed(i));
        }
        options = options.add(MenuOption::Plain);
        Self {
            menu: Menu::new(menu::Config {
                base: menu::BaseConfig::new("Run a debug script."),
                options,
                cancellability: Cancellable::new(),
            }),
        }
    }

    pub fn reset(&mut self) -> Result<(), ResetError> {
        self.menu.select(MenuOption::Suffixed(1))?;
        Ok(())
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
        game: &mut Game,
    ) -> Result<bool, TickError> {
        let running = self.menu.on_tick(tick)?;
        if !running {
            if let CancellableOutput::Accepted(option) = self.menu.selection() {
                let path = option.to_string();
                run_from(path, game)?;
            }
        }
        Ok(running)
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

fn read(path: impl AsRef<Path>) -> Result<Script, RunError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let script = serde_json::from_reader(reader)?;
    Ok(script)
}

pub fn run_from(
    path: impl AsRef<Path>,
    game: &mut Game,
) -> Result<(), RunError> {
    let script = read(path)?;
    script.run(game);
    Ok(())
}

pub fn run(game: &mut Game) -> Result<(), RunError> {
    run_from(DEFAULT_PATH, game)
}
