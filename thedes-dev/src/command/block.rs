use serde::{Deserialize, Serialize};
use thedes_domain::{geometry::Coord, stat::StatValue};

use crate::CommandContext;

use super::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandBlock {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    damage_player: Option<DamagePlayerCommand>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    heal_player: Option<HealPlayerCommand>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_monster_follow_limit_min: Option<SetMonsterFollowLimitMin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_monster_follow_limit_max: Option<SetMonsterFollowLimitMax>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_monster_follow_limit_peak: Option<SetMonsterFollowLimitPeak>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_monster_follow_period_min: Option<SetMonsterFollowPeriodMin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_monster_follow_period_max: Option<SetMonsterFollowPeriodMax>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    set_monster_follow_period_peak: Option<SetMonsterFollowPeriodPeak>,
}

impl Command for CommandBlock {
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()> {
        let Self {
            damage_player,
            heal_player,
            set_monster_follow_limit_min,
            set_monster_follow_limit_max,
            set_monster_follow_limit_peak,
            set_monster_follow_period_min,
            set_monster_follow_period_max,
            set_monster_follow_period_peak,
        } = self;

        if let Some(cmd) = damage_player {
            cmd.run(context)?;
        }
        if let Some(cmd) = heal_player {
            cmd.run(context)?;
        }

        if let Some(cmd) = set_monster_follow_limit_min {
            cmd.run(context)?;
        }
        if let Some(cmd) = set_monster_follow_limit_peak {
            cmd.run(context)?;
        }
        if let Some(cmd) = set_monster_follow_limit_max {
            cmd.run(context)?;
        }

        if let Some(cmd) = set_monster_follow_period_min {
            cmd.run(context)?;
        }
        if let Some(cmd) = set_monster_follow_period_peak {
            cmd.run(context)?;
        }
        if let Some(cmd) = set_monster_follow_period_max {
            cmd.run(context)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct DamagePlayerCommand {
    amount: StatValue,
}

impl Command for DamagePlayerCommand {
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()> {
        context.game.damage_player(self.amount);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct HealPlayerCommand {
    amount: StatValue,
}

impl Command for HealPlayerCommand {
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()> {
        context.game.heal_player(self.amount);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetMonsterFollowLimitMin {
    value: u32,
}

impl Command for SetMonsterFollowLimitMin {
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()> {
        context.event_distr_config.set_monster_follow_limit_min(self.value)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetMonsterFollowLimitPeak {
    value: u32,
}

impl Command for SetMonsterFollowLimitPeak {
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()> {
        context.event_distr_config.set_monster_follow_limit_peak(self.value)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetMonsterFollowLimitMax {
    value: u32,
}

impl Command for SetMonsterFollowLimitMax {
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()> {
        context.event_distr_config.set_monster_follow_limit_max(self.value)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetMonsterFollowPeriodMin {
    value: Coord,
}

impl Command for SetMonsterFollowPeriodMin {
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()> {
        context.event_distr_config.set_monster_follow_period_min(self.value)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetMonsterFollowPeriodPeak {
    value: Coord,
}

impl Command for SetMonsterFollowPeriodPeak {
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()> {
        context
            .event_distr_config
            .set_monster_follow_period_peak(self.value)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct SetMonsterFollowPeriodMax {
    value: Coord,
}

impl Command for SetMonsterFollowPeriodMax {
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()> {
        context.event_distr_config.set_monster_follow_period_max(self.value)?;
        Ok(())
    }
}
