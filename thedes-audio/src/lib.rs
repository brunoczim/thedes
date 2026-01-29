use std::{fmt, io::Cursor, sync::Arc};

use rodio::{OutputStream, Sink, StreamError, decoder::DecoderError};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AudioControllerType {
    Music,
}

impl fmt::Display for AudioControllerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Music => "music",
        })
    }
}

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

#[derive(Debug, Error)]
#[error("Failed to operate on audio controller type {controller_type}")]
pub struct ClientError<E> {
    pub controller_type: AudioControllerType,
    #[source]
    pub source: E,
}

impl<E> ClientError<E> {
    pub fn with(
        controller_type: AudioControllerType,
    ) -> impl FnOnce(E) -> Self {
        move |source| Self { controller_type, source }
    }
}

pub struct AudioController {
    stream: OutputStream,
    sink: Sink,
}

impl AudioController {
    pub fn connect() -> Result<Self, ConnectError> {
        let stream = rodio::OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::Sink::connect_new(&stream.mixer());
        Ok(Self { stream, sink })
    }

    pub fn play_now(&self, bytes: &'static [u8]) -> Result<(), PlayNowError> {
        let reader = Cursor::new(bytes);
        let source = rodio::Decoder::try_from(reader)?;

        self.sink.clear();
        self.sink.append(source);
        self.sink.set_volume(0.5);
        self.sink.play();

        Ok(())
    }
}

impl fmt::Debug for AudioController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioClientInner")
            .field("stream", &(&self.stream as *const _))
            .field("sink", &(&self.sink as *const _))
            .finish()
    }
}

#[derive(Debug)]
struct AudioClientInner {
    music: AudioController,
}

#[derive(Debug, Clone)]
pub struct AudioClient {
    inner: Arc<AudioClientInner>,
}

impl AudioClient {
    pub fn connect() -> Result<Self, ClientError<ConnectError>> {
        let music_controller = AudioController::connect()
            .map_err(ClientError::with(AudioControllerType::Music))?;
        Ok(Self {
            inner: Arc::new(AudioClientInner { music: music_controller }),
        })
    }

    pub fn controller(
        &self,
        controller_type: AudioControllerType,
    ) -> &AudioController {
        match controller_type {
            AudioControllerType::Music => &self.inner.music,
        }
    }

    pub fn play_now(
        &self,
        controller_type: AudioControllerType,
        bytes: &'static [u8],
    ) -> Result<(), ClientError<PlayNowError>> {
        self.controller(controller_type)
            .play_now(bytes)
            .map_err(ClientError::with(controller_type))
    }
}
