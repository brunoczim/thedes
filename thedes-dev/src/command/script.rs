use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{CommandContext, Error, ErrorKind};

use super::{Command, block::CommandBlock};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScriptTable {
    scripts: HashMap<char, Script>,
}

impl ScriptTable {
    pub const DEFAULT_PATH: &str = "thedes-cmd.json";

    pub async fn read_from(path: impl AsRef<Path>) -> Result<Self, Error> {
        let content = fs::read(path.as_ref())
            .await
            .map_err(Error::new_with_path(path.as_ref()))?;
        let script = serde_json::from_slice(&content[..])
            .map_err(Error::new_with_path(path.as_ref()))?;
        Ok(script)
    }

    pub async fn read() -> Result<Self, Error> {
        Self::read_from(Self::DEFAULT_PATH).await
    }

    pub async fn run_reading_from(
        path: impl AsRef<Path>,
        key: char,
        context: &mut CommandContext<'_, '_>,
    ) -> Result<(), Error> {
        let table = Self::read_from(path.as_ref()).await?;
        table.run(key, context).map_err(Error::with_path(path.as_ref()))?;
        Ok(())
    }

    pub async fn run_reading(
        key: char,
        context: &mut CommandContext<'_, '_>,
    ) -> Result<(), Error> {
        Self::run_reading_from(Self::DEFAULT_PATH, key, context).await
    }

    pub fn run(
        &self,
        key: char,
        context: &mut CommandContext,
    ) -> Result<(), Error> {
        match self.scripts.get(&key) {
            Some(script) => {
                if let Err(error) = script.run(context) {
                    tracing::error!("Development script failed");
                    tracing::error!("Error chain:");
                    for source in error.chain() {
                        tracing::error!("- {source}");
                    }
                    tracing::error!("Stack backtrace:");
                    tracing::error!("- {}", error.backtrace());
                }
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
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()> {
        match self {
            Self::Single(block) => block.run(context),
            Self::List(list) => {
                for block in list {
                    block.run(context)?
                }
                Ok(())
            },
        }
    }
}
