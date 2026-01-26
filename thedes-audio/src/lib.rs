use std::{fmt, io::Cursor, sync::Arc};

use rodio::{OutputStream, Sink, StreamError, decoder::DecoderError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Failed to manipulate audio stream")]
    Stream(#[from] StreamError),
}

#[derive(Debug, Error)]
pub enum PlayNowError {
    #[error("Failed to decode audio")]
    Decode(#[from] DecoderError),
}

struct AudioClientInner {
    stream: OutputStream,
    sink: Sink,
}

impl fmt::Debug for AudioClientInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioClientInner")
            .field("stream", &(&self.stream as *const _))
            .field("sink", &(&self.sink as *const _))
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct AudioClient {
    inner: Arc<AudioClientInner>,
}

impl AudioClient {
    pub fn connect() -> Result<Self, ConnectError> {
        let stream = rodio::OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::Sink::connect_new(&stream.mixer());
        Ok(Self { inner: Arc::new(AudioClientInner { stream, sink }) })
    }

    pub fn play_now(&self, bytes: &'static [u8]) -> Result<(), PlayNowError> {
        let reader = Cursor::new(bytes);
        let source = rodio::Decoder::try_from(reader)?;

        self.inner.sink.clear();
        self.inner.sink.append(source);
        self.inner.sink.set_volume(0.5);
        self.inner.sink.play();

        Ok(())
    }
}
