use std::{
    fs::File,
    io::{self, BufReader},
};

use num::rational::Ratio;
use serde::{Deserialize, Serialize};
use thedes_domain::{
    game::Game,
    time::{CircadianCycleStep, LunarPhase, Season},
};
use thiserror::Error;

pub const PATH: &str = "thedes-cmd.json";

#[derive(Debug, Error)]
pub enum Error {
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
    set_circadian_cycle_step: Option<SetCircadianCycleStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_day: Option<SetDay>,
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

fn read() -> Result<Script, Error> {
    let file = File::open(PATH)?;
    let reader = BufReader::new(file);
    let script = serde_json::from_reader(reader)?;
    Ok(script)
}

pub fn run(game: &mut Game) -> Result<(), Error> {
    let script = read()?;
    script.run(game);
    Ok(())
}
