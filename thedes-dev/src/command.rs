mod script;
mod block;

pub use script::ScriptTable;
use thedes_domain::game::Game;
use thedes_gen::event;

pub trait Command {
    fn run(&self, context: &mut CommandContext) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct CommandContext<'g, 'e> {
    pub game: &'g mut Game,
    pub event_distr_config: &'e mut event::DistrConfig,
}
