use anyhow::anyhow;
use futures::{
    future::BoxFuture,
    stream::{FuturesUnordered, StreamExt},
    TryStreamExt,
};
use gardiz::{bits::HalfExcess, direc::Direction};
use std::{collections::HashMap, io, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    select,
    sync::Mutex,
    task,
};
use tokio_util::sync::CancellationToken;

use crate::{
    domain::{Coord, HumanLocation, Map, Player, PlayerName, Vec2},
    error::Result,
    message::{
        self,
        ClientRequest,
        GetMapRequest,
        GetPlayerError,
        GetPlayerResponse,
        LoginError,
        LoginRequest,
        LoginResponse,
        MoveClientPlayerError,
        MoveClientPlayerResponse,
    },
};

#[derive(Debug)]
pub struct Server {
    jobs: FuturesUnordered<BoxFuture<'static, Result<()>>>,
    listener: TcpListener,
    shared: Arc<Shared>,
}

impl Server {
    pub async fn new<S>(
        bind_addr: S,
        cancel_token: CancellationToken,
    ) -> Result<Self>
    where
        S: ToSocketAddrs,
    {
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
            if let Some(client_conn) =
                ClientConn::new(stream, client_addr, shared).await?
            {
                client_conn.handle().await?;
            }
            Ok(())
        });
        let job: BoxFuture<_> = Box::pin(async move { handle.await? });
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
        let login: LoginRequest = message::receive(&mut stream).await?;
        let response = shared
            .state
            .lock()
            .await
            .connect_player(client_addr, &login.player_name);
        let is_success = response.result.is_ok();
        message::send(&mut stream, response).await?;

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
        let cleanup_result = self.cleanup().await;
        handle_result?;
        cleanup_result
    }

    async fn do_handle(&mut self) -> Result<()> {
        loop {
            select! {
                result = message::receive(&mut self.stream) => {
                    self.select_receive(result).await?;
                },
                () = self.shared.cancel_token.cancelled() => {
                    break Ok(());
                },
            }
        }
    }

    async fn cleanup(self) -> Result<()> {
        Ok(())
    }

    async fn select_receive(
        &mut self,
        result: Result<Option<ClientRequest>>,
    ) -> Result<()> {
        if let Some(client_request) = result? {
            match client_request {
                ClientRequest::GetPlayer(request) => {
                    let response = self
                        .shared
                        .state
                        .lock()
                        .await
                        .get_player(&request.player_name);
                    message::send(&mut self.stream, response).await?;
                },

                ClientRequest::MoveClientPlayer(request) => {
                    let response =
                        self.shared.state.lock().await.move_player(
                            &self.player_name,
                            request.direction,
                        )?;
                    message::send(&mut self.stream, response).await?;
                },

                ClientRequest::GetMapRequest(GetMapRequest) => {
                    let response = self.shared.state.lock().await.get_map();
                    message::send(&mut self.stream, response).await?;
                },
            }
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
    map: Map,
    players: HashMap<PlayerName, PlayerGameData>,
}

impl GameState {
    pub fn new() -> Self {
        Self { map: Map::default(), players: HashMap::new() }
    }

    pub fn connect_player(
        &mut self,
        client_addr: SocketAddr,
        player_name: &PlayerName,
    ) -> LoginResponse {
        if let Some(player_data) = self.players.get_mut(player_name) {
            if player_data.client_addr.is_some() {
                return LoginResponse { result: Err(LoginError::AlreadyIn) };
            }
            player_data.client_addr = Some(client_addr);
            LoginResponse { result: Ok(player_data.player.clone()) }
        } else {
            let player = Player {
                name: player_name.clone(),
                location: HumanLocation {
                    head: Vec2 {
                        x: Map::SIZE.x / 2 + 1,
                        y: Map::SIZE.y / 2 + 1,
                    },
                    facing: Direction::Up,
                },
            };
            self.players.insert(
                player_name.clone(),
                PlayerGameData {
                    client_addr: Some(client_addr),
                    player: player.clone(),
                },
            );
            LoginResponse { result: Ok(player) }
        }
    }

    pub fn get_player(
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

    pub fn move_player(
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
        self.map[player_data.player.location.head].player = None;
        self.map[player_data.player.location.pointer()].player = None;
        player_data.player.location = new_location;
        self.map[player_data.player.location.head].player =
            Some(player_data.player.clone());
        self.map[player_data.player.location.pointer()].player =
            Some(player_data.player.clone());
        Ok(MoveClientPlayerResponse { result: Ok(player_data.player.clone()) })
    }

    pub fn get_map(&self) -> Map {
        self.map.clone()
    }
}

impl Shared {
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self { cancel_token, state: Mutex::new(GameState::new()) }
    }
}
