use std::fmt;

use thedes_async_util::dyn_async_trait;
use thiserror::Error;
use tokio::io;

use crate::{color::Color, geometry::CoordPair};

pub mod native;
pub mod null;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Failed to format command")]
    Fmt(
        #[from]
        #[source]
        fmt::Error,
    ),
    #[error("Invalid command: {:#?}", .0)]
    InvalidCommand(Command),
    #[error(transparent)]
    Custom(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Enter,
    Leave,
    Clear,
    ResetBackground,
    ResetForeground,
    SetBackground(Color),
    SetForeground(Color),
    ShowCursor,
    HideCursor,
    MoveCursor(CoordPair),
    Write(char),
}

#[dyn_async_trait]
pub trait ScreenDevice: fmt::Debug + Send + Sync {
    fn send_raw(
        &mut self,
        commands: &mut (dyn Iterator<Item = Command> + Send + Sync),
    ) -> Result<(), Error>;

    async fn flush(&mut self) -> Result<(), Error>;
}

#[dyn_async_trait]
impl<'a, D> ScreenDevice for &'a mut D
where
    D: ScreenDevice + ?Sized,
{
    fn send_raw(
        &mut self,
        commands: &mut (dyn Iterator<Item = Command> + Send + Sync),
    ) -> Result<(), Error> {
        (**self).send_raw(commands)
    }

    async fn flush(&mut self) -> Result<(), Error> {
        (**self).flush().await
    }
}

#[dyn_async_trait]
impl<D> ScreenDevice for Box<D>
where
    D: ScreenDevice + ?Sized,
{
    fn send_raw(
        &mut self,
        commands: &mut (dyn Iterator<Item = Command> + Send + Sync),
    ) -> Result<(), Error> {
        (**self).send_raw(commands)
    }

    async fn flush(&mut self) -> Result<(), Error> {
        (**self).flush().await
    }
}

pub trait ScreenDeviceExt: ScreenDevice {
    fn send<I>(&mut self, iterable: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = Command>,
        I::IntoIter: Send + Sync,
    {
        self.send_raw(&mut iterable.into_iter())
    }
}

impl<S> ScreenDeviceExt for S where S: ScreenDevice {}
