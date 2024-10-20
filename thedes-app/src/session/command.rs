use std::{
    fs::File,
    io::{self, BufReader},
};

use serde::{Deserialize, Serialize};
use thedes_domain::game::Game;
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
enum Command {
    SetTime(SetTimeCommand),
    Set,
}

impl Command {
    fn run(&self, game: &mut Game) {
        match self {
            Self::SetTime(command) => command.run(game),
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
        game.set_time(self.stamp);
    }
}

fn read() -> Result<Command, Error> {
    let file = File::open(PATH)?;
    let reader = BufReader::new(file);
    let command = serde_json::from_reader(reader)?;
    Ok(command)
}

pub fn run(game: &mut Game) -> Result<(), Error> {
    let command = read()?;
    command.run(game);
    Ok(())
}
