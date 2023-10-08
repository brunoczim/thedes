use futures::{
    future::BoxFuture,
    stream::{FuturesUnordered, StreamExt},
    TryStreamExt,
};
use gardiz::direc::Direction;
use std::{collections::HashMap, io, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    select,
    sync::Mutex,
    task,
};
use tokio_util::sync::CancellationToken;

use crate::{
    domain::{Player, PlayerName, Vec2},
    error::Result,
    message::{
        self,
        ClientRequest,
        GetPlayerError,
        GetPlayerResponse,
        LoginError,
        LoginRequest,
        LoginResponse,
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
        match shared.connect_player(client_addr, &login.player_name).await {
            Ok(()) => {
                message::send(&mut stream, LoginResponse::Ok(())).await?;
                Ok(Some(Self {
                    stream,
                    client_addr,
                    player_name: login.player_name,
                    shared,
                }))
            },
            Err(error) => {
                message::send(&mut stream, LoginResponse::Err(error)).await?;
                Ok(None)
            },
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
                    let response =
                        self.shared.get_player(&request.player_name).await?;
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
    players: Mutex<HashMap<PlayerName, PlayerGameData>>,
}

impl Shared {
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self { cancel_token, players: Mutex::new(HashMap::new()) }
    }

    pub async fn connect_player(
        &self,
        client_addr: SocketAddr,
        player_name: &PlayerName,
    ) -> LoginResponse {
        let mut players = self.players.lock().await;
        if let Some(player_data) = players.get_mut(player_name) {
            if player_data.client_addr.is_some() {
                Err(LoginError::AlreadyIn)?;
            }
            player_data.client_addr = Some(client_addr);
        } else {
            players.insert(
                player_name.clone(),
                PlayerGameData {
                    client_addr: Some(client_addr),
                    player: Player {
                        name: player_name.clone(),
                        location: Vec2 { x: 0, y: 0 },
                        pointer: Direction::Up,
                    },
                },
            );
        }
        Ok(())
    }

    pub async fn get_player(
        &self,
        player_name: &PlayerName,
    ) -> GetPlayerResponse {
        let players = self.players.lock().await;
        let player_data = players.get(player_name).ok_or_else(|| {
            GetPlayerError::UnknownPlayer(player_name.clone())
        })?;
        if player_data.client_addr.is_none() {
            Err(GetPlayerError::PlayerLoggedOff(player_name.clone()))?;
        }
        Ok(player_data.player.clone())
    }
}
