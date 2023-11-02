use std::io;

use anyhow::anyhow;
use bincode::{DefaultOptions, Options};
use gardiz::direc::Direction;
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    domain::{GameSnapshot, Map, Player, PlayerName},
    error::Result,
};

pub const MAX_LENGTH: u32 = 1 << 28;

pub const MAGIC_BEGIN: u8 = 0x_c7;

pub const MAGIC_END: u8 = 0x_b3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoginRequest {
    pub player_name: PlayerName,
}

#[derive(Debug, Clone, Error, serde::Serialize, serde::Deserialize)]
pub enum LoginError {
    #[error("player already in (check player name)")]
    AlreadyIn,
    #[error("invalid player name {}", .0)]
    InvalidName(PlayerName),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoginResponse {
    pub result: Result<GameSnapshot, LoginError>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ClientRequest {
    MoveClientPlayer(MoveClientPlayerRequest),
    GetSnapshotRequest(GetSnapshotRequest),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetPlayerRequest {
    pub player_name: PlayerName,
}

#[derive(Debug, Clone, Error, serde::Serialize, serde::Deserialize)]
pub enum GetPlayerError {
    #[error("unknown player {}", .0)]
    UnknownPlayer(PlayerName),
    #[error("player {} logged of", .0)]
    PlayerLoggedOff(PlayerName),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetPlayerResponse {
    pub result: Result<Player, GetPlayerError>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MoveClientPlayerRequest {
    pub direction: Direction,
}

#[derive(Debug, Clone, Error, serde::Serialize, serde::Deserialize)]
pub enum MoveClientPlayerError {
    #[error("player cannot be moved because it would violate map limits")]
    OffLimits,
    #[error("player cannot be moved because it would collide")]
    Collision,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MoveClientPlayerResponse {
    pub result: Result<(), MoveClientPlayerError>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetSnapshotRequest;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetSnapshotResponse {
    pub snapshot: GameSnapshot,
}

pub fn bincode_options() -> impl Options + Send + Sync + 'static {
    DefaultOptions::new().with_little_endian().reject_trailing_bytes()
}

pub async fn receive<M>(stream: &mut TcpStream) -> Result<M>
where
    M: for<'de> serde::Deserialize<'de>,
{
    loop {
        if let Some(message) = try_receive(&mut *stream).await? {
            break Ok(message);
        }
    }
}

async fn patient_read(
    stream: &mut TcpStream,
    mut buf: &mut [u8],
) -> Result<()> {
    while buf.len() > 0 {
        stream.readable().await?;
        let count = stream.read(&mut *buf).await?;
        if count == 0 {
            Err(io::Error::from(io::ErrorKind::ConnectionAborted))?;
        }
        buf = &mut buf[count ..];
    }
    Ok(())
}

pub async fn try_receive<M>(stream: &mut TcpStream) -> Result<Option<M>>
where
    M: for<'de> serde::Deserialize<'de>,
{
    let mut magic_begin_buf = [0; 1];
    patient_read(stream, &mut magic_begin_buf).await?;
    let maybe_magic_begin = u8::from_le_bytes(magic_begin_buf);
    if maybe_magic_begin == MAGIC_BEGIN {
        let mut length_buf = [0; 4];
        patient_read(stream, &mut length_buf[..]).await?;
        let length = u32::from_le_bytes(length_buf);
        if length > MAX_LENGTH {
            Err(anyhow!(
                "maximum message length is {} but found {}",
                MAX_LENGTH,
                length
            ))?;
        }
        let Ok(usize_length) = usize::try_from(length) else {
            Err(anyhow!("server cannot address message of length {}", length))?
        };
        let mut message_buf = vec![0; usize_length];
        patient_read(stream, &mut message_buf[..]).await?;
        let message = bincode::deserialize(&message_buf[..])?;
        let mut magic_end_buf = [0; 1];
        patient_read(stream, &mut magic_end_buf[..]).await?;
        let maybe_magic_end = u8::from_le_bytes(magic_end_buf);
        if maybe_magic_end != MAGIC_END {
            Err(anyhow!(
                "message must end with magic end number {}, found {}",
                MAGIC_END,
                maybe_magic_end
            ))?;
        }
        Ok(Some(message))
    } else {
        Ok(None)
    }
}

pub async fn send<M>(stream: &mut TcpStream, message: M) -> Result<()>
where
    M: serde::Serialize,
{
    let message_buf = bincode::serialize(&message)?;
    let magic_begin_buf = MAGIC_BEGIN.to_le_bytes();
    stream.write_all(&magic_begin_buf[..]).await?;
    let usize_length = message_buf.len();
    let Some(length) =
        u32::try_from(usize_length).ok().filter(|length| *length <= MAX_LENGTH)
    else {
        Err(anyhow!(
            "maximum message length is {} but found {}",
            MAX_LENGTH,
            usize_length
        ))?
    };
    let length_buf = length.to_le_bytes();
    stream.write_all(&length_buf[..]).await?;
    stream.write_all(&message_buf[..]).await?;
    let magic_end_buf = MAGIC_END.to_le_bytes();
    stream.write_all(&magic_end_buf[..]).await?;
    Ok(())
}
