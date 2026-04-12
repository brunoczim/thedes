use std::{borrow::Cow, fmt, io};

use thiserror::Error;

pub mod null;
pub mod native;
pub mod mock;

#[derive(Debug, Error)]
pub enum PlayNowError {
    #[error("Failed to decode audio: {0}")]
    Decode(String),
    #[error("I/O error playing audio")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum SetVolumeError {
    #[error("I/O error setting volume")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ClearSinkError {
    #[error("I/O error clearing the sink")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum PauseSinkError {
    #[error("I/O error pausing the sink")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ResumeSinkError {
    #[error("I/O error resuming the sink")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum CheckPlayStatusError {
    #[error("I/O error checking playing status")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum OpenSinkError {
    #[error("I/O error opening sink")]
    Io(#[from] io::Error),
    #[error("Failed to manipulate audio stream: {0}")]
    Stream(String),
}

pub trait AudioDevice: fmt::Debug + Send + Sync {
    fn open_sink(&mut self) -> Result<Box<dyn AudioSinkDevice>, OpenSinkError>;
}

impl<'a, A> AudioDevice for &'a mut A
where
    A: AudioDevice + ?Sized,
{
    fn open_sink(&mut self) -> Result<Box<dyn AudioSinkDevice>, OpenSinkError> {
        (**self).open_sink()
    }
}

impl<A> AudioDevice for Box<A>
where
    A: AudioDevice + ?Sized,
{
    fn open_sink(&mut self) -> Result<Box<dyn AudioSinkDevice>, OpenSinkError> {
        (**self).open_sink()
    }
}

pub trait AudioSinkDevice: fmt::Debug + Send + Sync {
    fn play_now(
        &mut self,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), PlayNowError>;

    fn set_volume(&mut self, volume: f32) -> Result<(), SetVolumeError>;

    fn pause(&mut self) -> Result<(), PauseSinkError>;

    fn resume(&mut self) -> Result<(), ResumeSinkError>;

    fn clear(&mut self) -> Result<(), ClearSinkError>;

    fn is_playing(&self) -> Result<bool, CheckPlayStatusError>;
}

impl<'a, S> AudioSinkDevice for &'a mut S
where
    S: AudioSinkDevice + ?Sized,
{
    fn play_now(
        &mut self,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), PlayNowError> {
        (**self).play_now(bytes)
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), SetVolumeError> {
        (**self).set_volume(volume)
    }

    fn pause(&mut self) -> Result<(), PauseSinkError> {
        (**self).pause()
    }

    fn resume(&mut self) -> Result<(), ResumeSinkError> {
        (**self).resume()
    }

    fn clear(&mut self) -> Result<(), ClearSinkError> {
        (**self).clear()
    }

    fn is_playing(&self) -> Result<bool, CheckPlayStatusError> {
        (**self).is_playing()
    }
}

impl<S> AudioSinkDevice for Box<S>
where
    S: AudioSinkDevice + ?Sized,
{
    fn play_now(
        &mut self,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), PlayNowError> {
        (**self).play_now(bytes)
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), SetVolumeError> {
        (**self).set_volume(volume)
    }

    fn pause(&mut self) -> Result<(), PauseSinkError> {
        (**self).pause()
    }

    fn resume(&mut self) -> Result<(), ResumeSinkError> {
        (**self).resume()
    }

    fn clear(&mut self) -> Result<(), ClearSinkError> {
        (**self).clear()
    }

    fn is_playing(&self) -> Result<bool, CheckPlayStatusError> {
        (**self).is_playing()
    }
}
