use serde::{Deserialize, Serialize};
use thedes_domain::{game::Game, stat::StatValue};

use super::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandBlock {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    damage_player: Option<DamagePlayerCommand>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    heal_player: Option<HealPlayerCommand>,
}

impl Command for CommandBlock {
    fn run(&self, game: &mut Game) {
        let Self { damage_player: damage, heal_player: heal } = self;

        if let Some(cmd) = damage {
            cmd.run(game);
        }

        if let Some(cmd) = heal {
            cmd.run(game);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct DamagePlayerCommand {
    amount: StatValue,
}

impl Command for DamagePlayerCommand {
    fn run(&self, game: &mut Game) {
        game.damage_player(self.amount);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct HealPlayerCommand {
    amount: StatValue,
}

impl Command for HealPlayerCommand {
    fn run(&self, game: &mut Game) {
        game.heal_player(self.amount);
    }
}
