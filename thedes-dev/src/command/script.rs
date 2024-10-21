use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};
use thedes_domain::game::Game;

use crate::{Error, ErrorKind};

use super::{block::CommandBlock, Command};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScriptTable {
    scripts: HashMap<char, Script>,
}

impl ScriptTable {
    pub const DEFAULT_PATH: &str = "thedes-cmd.json";

    pub fn read_from(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = File::open(path.as_ref())
            .map_err(Error::new_with_path(path.as_ref()))?;
        let reader = BufReader::new(file);
        let script = serde_json::from_reader(reader)
            .map_err(Error::new_with_path(path.as_ref()))?;
        Ok(script)
    }

    pub fn read() -> Result<Self, Error> {
        Self::read_from(Self::DEFAULT_PATH)
    }

    pub fn run_reading_from(
        path: impl AsRef<Path>,
        key: char,
        game: &mut Game,
    ) -> Result<(), Error> {
        let table = Self::read_from(path.as_ref())
            .map_err(Error::with_path(path.as_ref()))?;
        table.run(key, game).map_err(Error::with_path(path.as_ref()))?;
        Ok(())
    }

    pub fn run_reading(key: char, game: &mut Game) -> Result<(), Error> {
        Self::run_reading_from(Self::DEFAULT_PATH, key, game)
    }

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

impl Command for Script {
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
