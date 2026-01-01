mod script;
mod block;

pub use script::ScriptTable;
use thedes_domain::game::Game;

pub trait Command {
    fn run(&self, game: &mut Game);
}
