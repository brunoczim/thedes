pub mod error;

use async_trait::async_trait;
pub use error::Result;
use thedes_common::player::{self, Player};

#[async_trait]
pub trait Server {
    async fn load_player(&mut self, id: player::Id) -> Result<Player>;
}
