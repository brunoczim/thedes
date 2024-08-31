use std::{any::type_name, fmt};

use rodio::{OutputStream, PlayError, Sink, StreamError};
use thiserror::Error;

use crate::signal::Signal;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to open output stream")]
    OutputStream(#[source] StreamError),
    #[error("Error creating a sink")]
    Sink(#[source] PlayError),
}

pub struct Player {
    device: rodio::OutputStreamHandle,
    sink: Sink,
}

impl fmt::Debug for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("device", &[] as &[u8; 0])
            .field("sink", &[] as &[u8; 0])
            .finish()
    }
}

impl Player {
    pub fn new() -> Result<Self, Error> {
        let (_, handle) =
            OutputStream::try_default().map_err(Error::OutputStream)?;
        let sink = Sink::try_new(&handle).map_err(Error::Sink)?;
        Ok(Self { device: handle, sink })
    }

    pub fn set_signal_from_start<S>(&self, signal: S)
    where
        S: Signal<f32, Sample = f32> + Send + Sync + 'static,
    {
        self.set_signal_from(0.0, signal)
    }

    pub fn set_signal_from<S>(&self, start: f32, signal: S)
    where
        S: Signal<f32, Sample = f32> + Send + Sync + 'static,
    {
        // TODO
    }
}

#[derive(Debug, Clone)]
struct Adapter<S> {
    signal: S,
    // TODO
}
