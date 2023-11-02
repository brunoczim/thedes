use anyhow::anyhow;
use futures::{
    future::BoxFuture,
    stream::{FuturesUnordered, StreamExt},
    TryStreamExt,
};
use gardiz::{coord::Vec2, direc::Direction};
use rand::{
    rngs::{OsRng, StdRng},
    Rng,
    RngCore,
    SeedableRng,
};
use std::{collections::HashMap, io, net::SocketAddr, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    select,
    sync::Mutex,
    task,
};
use tokio_util::sync::CancellationToken;

use crate::{
    domain::{GameSnapshot, HumanLocation, Map, Player, PlayerName},
    error::Result,
    message::{
        self,
        ClientRequest,
        GetPlayerError,
        GetPlayerResponse,
        GetSnapshotRequest,
        GetSnapshotResponse,
        LoginError,
        LoginRequest,
        LoginResponse,
        LogoutNotice,
        MoveClientPlayerError,
        MoveClientPlayerResponse,
    },
};

type GameRng = StdRng;

#[derive(Debug)]
pub struct Server {
    jobs: FuturesUnordered<BoxFuture<'static, Result<()>>>,
    listener: TcpListener,
    shared: Arc<Shared>,
}

impl Server {
    pub async fn new(
        bind_addr: SocketAddr,
        cancel_token: CancellationToken,
    ) -> Result<Self> {
        tracing::info!("Binding to {}", bind_addr);
        Ok(Self {
            jobs: FuturesUnordered::new(),
            listener: TcpListener::bind(bind_addr).await?,
            shared: Arc::new(Shared::new(cancel_token)),
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let run_result = self.do_run().await;
        let cleanup_result = self.cleanup().await;
        run_result?;
        cleanup_result
    }

    async fn do_run(&mut self) -> Result<()> {
        loop {
            select! {
                result = self.listener.accept() => {
                    self.select_accept(result).await?;
                },
                Some(result) = self.jobs.next() => {
                    result?
                },
                () = self.shared.cancel_token.cancelled() => {
                    break Ok(());
                },
            }
        }
    }

    async fn cleanup(self) -> Result<()> {
        self.jobs.try_for_each(|_| async { Ok(()) }).await
    }

    async fn select_accept(
        &mut self,
        result: io::Result<(TcpStream, SocketAddr)>,
    ) -> Result<()> {
        let (stream, client_addr) = result?;
        let shared = self.shared.clone();
        let handle = task::spawn(async move {
            let task = async move {
                if let Some(client_conn) =
                    ClientConn::new(stream, client_addr, shared).await?
                {
                    client_conn.handle().await?;
                }
                Result::<_>::Ok(())
            };

            if let Err(error) = task.await {
                tracing::error!(
                    "Connection with client {} failed: {}",
                    client_addr,
                    error
                )
            }
        });
        let job: BoxFuture<_> = Box::pin(async move {
            handle.await?;
            Ok(())
        });
        self.jobs.push(job);
        Ok(())
    }
}

#[derive(Debug)]
struct ClientConn {
    stream: TcpStream,
    client_addr: SocketAddr,
    player_name: PlayerName,
    shared: Arc<Shared>,
}

impl ClientConn {
    pub async fn new(
        mut stream: TcpStream,
        client_addr: SocketAddr,
        shared: Arc<Shared>,
    ) -> Result<Option<Self>> {
        tracing::info!("Establishing connection to {}", client_addr);

        let login: LoginRequest = message::receive(&mut stream).await?;

        tracing::debug!(
            "Received login request from {} with player={}",
            client_addr,
            login.player_name,
        );

        let response = shared
            .state
            .lock()
            .await
            .exec_login(client_addr, &login.player_name);

        let is_success = response.result.is_ok();

        tracing::debug!(
            "Login of {} with player={} is successful? {:?}",
            client_addr,
            login.player_name,
            is_success,
        );

        message::send(&mut stream, response).await?;

        tracing::debug!(
            "Login response to {} with player={} was sent",
            client_addr,
            login.player_name,
        );

        if is_success {
            Ok(Some(Self {
                stream,
                client_addr,
                player_name: login.player_name,
                shared,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn handle(mut self) -> Result<()> {
        let handle_result = self.do_handle().await;
        tracing::info!("Disconnecting {}", self.client_addr);
        let cleanup_result = self.cleanup().await;
        handle_result?;
        cleanup_result
    }

    async fn do_handle(&mut self) -> Result<()> {
        while self
            .shared
            .state
            .lock()
            .await
            .is_player_connected(&self.player_name)?
        {
            select! {
                result = message::receive(&mut self.stream) => {
                    self.select_receive(result).await?;
                },
                () = self.shared.cancel_token.cancelled() => {
                    break;
                },
            }
        }
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<()> {
        let shutdown_result = self.stream.shutdown().await;
        self.shared.state.lock().await.log_player_out(&self.player_name)?;
        shutdown_result?;
        Ok(())
    }

    async fn select_receive(
        &mut self,
        result: Result<ClientRequest>,
    ) -> Result<()> {
        let client_request = result?;
        match client_request {
            ClientRequest::MoveClientPlayer(request) => {
                let response =
                    self.shared.state.lock().await.exec_move_player(
                        &self.player_name,
                        request.direction,
                    )?;
                message::send(&mut self.stream, response).await?;
            },

            ClientRequest::GetSnapshotRequest(GetSnapshotRequest) => {
                let response =
                    self.shared.state.lock().await.exec_get_snapshot();
                message::send(&mut self.stream, response).await?;
            },

            ClientRequest::LogoutNotice(LogoutNotice) => {},
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct PlayerGameData {
    client_addr: Option<SocketAddr>,
    player: Player,
}

#[derive(Debug)]
struct Shared {
    cancel_token: CancellationToken,
    state: Mutex<GameState>,
}

#[derive(Debug)]
struct GameState {
    rng: GameRng,
    map: Map,
    players: HashMap<PlayerName, PlayerGameData>,
}

impl GameState {
    pub fn new() -> Self {
        let mut seed: <GameRng as SeedableRng>::Seed = Default::default();
        OsRng::default().fill_bytes(&mut seed);
        let rng = GameRng::from_seed(seed);
        Self { rng, map: Map::default(), players: HashMap::new() }
    }

    pub fn gen_snapshot(&self) -> GameSnapshot {
        GameSnapshot {
            map: self.map.clone(),
            players: self
                .players
                .iter()
                .map(|(key, data)| (key.clone(), data.player.clone()))
                .collect(),
        }
    }

    fn internal_get_player(
        &self,
        player_name: &PlayerName,
    ) -> Result<&PlayerGameData> {
        self.players.get(player_name).ok_or_else(|| {
            anyhow!(
                "player with name {} should exist, but it doesn't",
                player_name
            )
        })
    }

    fn internal_get_player_mut(
        &mut self,
        player_name: &PlayerName,
    ) -> Result<&mut PlayerGameData> {
        self.players.get_mut(player_name).ok_or_else(|| {
            anyhow!(
                "player with name {} should exist, but it doesn't",
                player_name
            )
        })
    }

    pub fn is_player_connected(
        &self,
        player_name: &PlayerName,
    ) -> Result<bool> {
        let is_connected =
            self.internal_get_player(player_name)?.client_addr.is_some();
        Ok(is_connected)
    }

    pub fn log_player_out(&mut self, player_name: &PlayerName) -> Result<()> {
        self.internal_get_player_mut(player_name)?.client_addr = None;
        Ok(())
    }

    pub fn exec_login(
        &mut self,
        client_addr: SocketAddr,
        player_name: &PlayerName,
    ) -> LoginResponse {
        let player_data = if let Some(player_data) =
            self.players.get_mut(player_name)
        {
            if player_data.client_addr.is_some() {
                return LoginResponse { result: Err(LoginError::AlreadyIn) };
            }
            player_data.client_addr = Some(client_addr);
            player_data
        } else {
            let location = loop {
                let location = HumanLocation {
                    head: Vec2 {
                        x: self
                            .rng
                            .gen_range(Map::SIZE.x / 4 .. Map::SIZE.x / 4 * 3),
                        y: self
                            .rng
                            .gen_range(Map::SIZE.y / 4 .. Map::SIZE.y / 4 * 3),
                    },
                    facing: Direction::Up,
                };
                if !self.map[location.head]
                    .player
                    .as_ref()
                    .map_or(false, |player| player.name != *player_name)
                    && !self.map[location.pointer()]
                        .player
                        .as_ref()
                        .map_or(false, |player| player.name != *player_name)
                {
                    break location;
                }
            };

            let player = Player { name: player_name.clone(), location };
            self.players.entry(player_name.clone()).or_insert(PlayerGameData {
                client_addr: Some(client_addr),
                player: player.clone(),
            })
        };
        self.map[player_data.player.location.head].player =
            Some(player_data.player.clone());
        self.map[player_data.player.location.pointer()].player =
            Some(player_data.player.clone());
        LoginResponse { result: Ok(self.gen_snapshot()) }
    }

    pub fn exec_get_player(
        &mut self,
        player_name: &PlayerName,
    ) -> GetPlayerResponse {
        let Some(player_data) = self.players.get(player_name) else {
            return GetPlayerResponse {
                result: Err(GetPlayerError::UnknownPlayer(player_name.clone())),
            };
        };
        if player_data.client_addr.is_none() {
            return GetPlayerResponse {
                result: Err(GetPlayerError::PlayerLoggedOff(
                    player_name.clone(),
                )),
            };
        }
        GetPlayerResponse { result: Ok(player_data.player.clone()) }
    }

    pub fn exec_move_player(
        &mut self,
        player_name: &PlayerName,
        direction: Direction,
    ) -> Result<MoveClientPlayerResponse> {
        let player_data =
            self.players.get_mut(player_name).ok_or_else(|| {
                anyhow!(
                    "player {} should be present, but it is not",
                    player_name
                )
            })?;
        let new_location = if player_data.player.location.facing == direction {
            let Some(new_head) = player_data
                .player
                .location
                .head
                .checked_move(direction)
                .filter(|new_head| {
                    new_head.x < Map::SIZE.x && new_head.y < Map::SIZE.y
                })
            else {
                return Ok(MoveClientPlayerResponse {
                    result: Err(MoveClientPlayerError::OffLimits),
                });
            };
            HumanLocation { head: new_head, facing: direction }
        } else {
            HumanLocation {
                head: player_data.player.location.head,
                facing: direction,
            }
        };
        if new_location
            .checked_pointer()
            .filter(|new_pointer| {
                new_pointer.x < Map::SIZE.x && new_pointer.y < Map::SIZE.y
            })
            .is_none()
        {
            return Ok(MoveClientPlayerResponse {
                result: Err(MoveClientPlayerError::OffLimits),
            });
        }

        if self.map[new_location.head]
            .player
            .as_ref()
            .map_or(false, |player| player.name != *player_name)
            || self.map[new_location.pointer()]
                .player
                .as_ref()
                .map_or(false, |player| player.name != *player_name)
        {
            return Ok(MoveClientPlayerResponse {
                result: Err(MoveClientPlayerError::Collision),
            });
        }

        self.map[player_data.player.location.head].player = None;
        self.map[player_data.player.location.pointer()].player = None;
        player_data.player.location = new_location;
        self.map[player_data.player.location.head].player =
            Some(player_data.player.clone());
        self.map[player_data.player.location.pointer()].player =
            Some(player_data.player.clone());
        Ok(MoveClientPlayerResponse { result: Ok(()) })
    }

    pub fn exec_get_snapshot(&self) -> GetSnapshotResponse {
        GetSnapshotResponse { snapshot: self.gen_snapshot() }
    }
}

impl Shared {
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self { cancel_token, state: Mutex::new(GameState::new()) }
    }
}
