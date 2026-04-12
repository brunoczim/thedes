use std::borrow::Cow;

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
    Box::new(NullAudioDevice)
}

#[derive(Debug, Clone, Copy)]
struct NullAudioDevice;

impl AudioDevice for NullAudioDevice {
    fn open_sink(&mut self) -> Result<Box<dyn AudioSinkDevice>, OpenSinkError> {
        Ok(Box::new(NullAudioSinkDevice))
    }
}

#[derive(Debug, Clone, Copy)]
struct NullAudioSinkDevice;

impl AudioSinkDevice for NullAudioSinkDevice {
    fn play_now(
        &mut self,
        _bytes: Cow<'static, [u8]>,
    ) -> Result<(), PlayNowError> {
        Ok(())
    }

    fn set_volume(&mut self, _volume: f32) -> Result<(), SetVolumeError> {
        Ok(())
    }

    fn pause(&mut self) -> Result<(), PauseSinkError> {
        Ok(())
    }

    fn resume(&mut self) -> Result<(), ResumeSinkError> {
        Ok(())
    }

    fn clear(&mut self) -> Result<(), ClearSinkError> {
        Ok(())
    }

    fn is_playing(&self) -> Result<bool, CheckPlayStatusError> {
        Ok(false)
    }
}
