use std::{borrow::Cow, fmt, io::Cursor};

use crate::audio::device::{
    AudioDevice,
    AudioSinkDevice,
    CheckPlayStatusError,
    ClearSinkError,
    OpenSinkError,
    PauseSinkError,
    PlayNowError,
    ResumeSinkError,
    SetVolumeError,
};

pub fn open() -> Box<dyn AudioDevice> {
    Box::new(NativeAudioDevice { handle: None })
}

struct NativeAudioDevice {
    handle: Option<rodio::MixerDeviceSink>,
}

impl AudioDevice for NativeAudioDevice {
    fn open_sink(&mut self) -> Result<Box<dyn AudioSinkDevice>, OpenSinkError> {
        let handle = match self.handle.take() {
            Some(stream) => stream,
            None => {
                let mut stream = rodio::DeviceSinkBuilder::open_default_sink()
                    .map_err(|e| OpenSinkError::Stream(e.to_string()))?;
                stream.log_on_drop(false);
                stream
            },
        };
        let player = rodio::Player::connect_new(&handle.mixer());
        self.handle = Some(handle);
        Ok(Box::new(NativeAudioSinkDevice { inner: player }))
    }
}

impl fmt::Debug for NativeAudioDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeAudioDevice")
            .field("stream", &(&self.handle as *const _))
            .finish()
    }
}

struct NativeAudioSinkDevice {
    inner: rodio::Player,
}

impl fmt::Debug for NativeAudioSinkDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeAudioSink")
            .field("sink", &(&self.inner as *const _))
            .finish()
    }
}

impl AudioSinkDevice for NativeAudioSinkDevice {
    fn play_now(
        &mut self,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), PlayNowError> {
        let reader = Cursor::new(bytes);
        let source = rodio::Decoder::try_from(reader)
            .map_err(|e| PlayNowError::Decode(e.to_string()))?;

        self.inner.clear();
        self.inner.append(source);
        self.inner.play();

        Ok(())
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), SetVolumeError> {
        self.inner.set_volume(volume);
        Ok(())
    }

    fn pause(&mut self) -> Result<(), PauseSinkError> {
        self.inner.pause();
        Ok(())
    }

    fn resume(&mut self) -> Result<(), ResumeSinkError> {
        self.inner.play();
        Ok(())
    }

    fn clear(&mut self) -> Result<(), ClearSinkError> {
        self.inner.clear();
        Ok(())
    }

    fn is_playing(&self) -> Result<bool, CheckPlayStatusError> {
        Ok(!self.inner.empty())
    }
}

impl Drop for NativeAudioSinkDevice {
    fn drop(&mut self) {
        self.inner.clear();
    }
}
