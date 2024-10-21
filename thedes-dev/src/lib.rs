use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use num::rational::Ratio;
use serde::{Deserialize, Serialize};
use thedes_domain::{
    game::Game,
    time::{CircadianCycleStep, LunarPhase, Season},
};
use thiserror::Error;

pub const DEFAULT_PATH: &str = "thedes-cmd.json";

#[derive(Debug, Error)]
pub struct Error {
    path: Option<PathBuf>,
    #[source]
    kind: ErrorKind,
}

impl Error {
    fn new(kind: impl Into<ErrorKind>) -> Self {
        Self { path: None, kind: kind.into() }
    }

    fn new_with_path<E>(path: impl Into<PathBuf>) -> impl FnOnce(E) -> Self
    where
        E: Into<ErrorKind>,
    {
        |kind| Self::with_path(path)(Self::new(kind))
    }

    fn with_path(path: impl Into<PathBuf>) -> impl FnOnce(Self) -> Self {
        |this| Self { path: Some(path.into()), ..this }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(path) = &self.path {
            write!(f, ", path: {}", path.display())?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Failed reading development scripts file")]
    Read(
        #[source]
        #[from]
        io::Error,
    ),
    #[error("Failed decoding script")]
    Decode(
        #[from]
        #[source]
        serde_json::Error,
    ),
    #[error("Unknown key {:?}", .0)]
    UnknownKey(char),
}

pub fn read_table_from(path: impl AsRef<Path>) -> Result<ScriptTable, Error> {
    let file = File::open(path.as_ref())
        .map_err(Error::new_with_path(path.as_ref()))?;
    let reader = BufReader::new(file);
    let script = serde_json::from_reader(reader)
        .map_err(Error::new_with_path(path.as_ref()))?;
    Ok(script)
}

pub fn read_table() -> Result<ScriptTable, Error> {
    read_table_from(DEFAULT_PATH)
}

pub fn run_from(
    path: impl AsRef<Path>,
    key: char,
    game: &mut Game,
) -> Result<(), Error> {
    let table = read_table_from(path.as_ref())
        .map_err(Error::with_path(path.as_ref()))?;
    table.run(key, game).map_err(Error::with_path(path.as_ref()))?;
    Ok(())
}

pub fn run(key: char, game: &mut Game) -> Result<(), Error> {
    run_from(DEFAULT_PATH, key, game)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScriptTable {
    scripts: HashMap<char, Script>,
}

impl ScriptTable {
    pub fn run(&self, key: char, game: &mut Game) -> Result<(), Error> {
        match self.scripts.get(&key) {
            Some(script) => {
                script.run(game);
                Ok(())
            },
            None => Err(Error::new(ErrorKind::UnknownKey(key))),
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
