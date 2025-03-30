use std::{fmt, time::Duration};

use thiserror::Error;
use tokio::io;

use crate::event::InternalEvent;

pub mod native;
pub mod null;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub trait InputDevice: fmt::Debug + Send + Sync {
    fn blocking_read(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<InternalEvent>, Error>;
}

impl<'a, T> InputDevice for &'a mut T
where
    T: InputDevice + ?Sized,
{
    fn blocking_read(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<InternalEvent>, Error> {
        (**self).blocking_read(timeout)
    }
}

impl<T> InputDevice for Box<T>
where
    T: InputDevice + ?Sized,
{
    fn blocking_read(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<InternalEvent>, Error> {
        (**self).blocking_read(timeout)
    }
}
