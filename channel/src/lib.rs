use async_trait::async_trait;
use thedes_common::{
    player::{self, Player},
    Result,
};

#[async_trait]
pub trait ServerHandle {
    type SessionHandle: ServerSessionHandle;

    async fn open(&self, player: player::Id) -> Result<Self::SessionHandle>;
}

#[async_trait]
pub trait ServerSessionHandle {
    async fn load_player(&mut self, id: player::Id) -> Result<Player>;

    async fn update_main_player(&mut self, data: &player::Data) -> Result<()>;
}
