use std::collections::HashMap;

use thiserror::Error;

use crate::audio::device::{AudioDevice, AudioSinkDevice, SetVolumeError};

pub mod device;

#[derive(Debug, Error)]
pub enum PlayNowError {
    #[error("Device failed to play")]
    DevicePlayNow(#[from] device::PlayNowError),
    #[error("Device failed to open sink")]
    DeviceOpenSink(#[from] device::OpenSinkError),
    #[error("Device failed to set volume")]
    DeviceSetVolume(#[from] device::SetVolumeError),
}

#[derive(Debug)]
pub(crate) struct OpenResources {
    pub device: Box<dyn AudioDevice>,
}

#[derive(Debug)]
pub struct Config {
    _private: (),
}

impl Config {
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub(crate) fn open(self, resources: OpenResources) -> AudioController {
        AudioController { device: resources.device, sinks: HashMap::new() }
    }
}

#[derive(Debug)]
struct AudioSinkGroup {
    sinks: Vec<Box<dyn AudioSinkDevice>>,
}

impl AudioSinkGroup {
    fn new() -> Self {
        Self { sinks: Vec::new() }
    }

    fn set_volume(&mut self, volume: u8) -> Result<(), SetVolumeError> {
        let volume = f32::from(volume) / 255.0;
        for device in &mut self.sinks {
            device.set_volume(volume)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct AudioController {
    device: Box<dyn AudioDevice>,
    sinks: HashMap<String, AudioSinkGroup>,
}

impl AudioController {
    pub fn set_volume(
        &mut self,
        key: &str,
        volume: u8,
    ) -> Result<(), SetVolumeError> {
        self.with_sink(key, |group, _| group.set_volume(volume))
    }

    pub fn play_now(
        &mut self,
        key: &str,
        bytes: &'static [u8],
    ) -> Result<(), PlayNowError> {
        self.with_sink(key, |group, device| {
            let mut sink = device.open_sink()?;
            sink.play_now(bytes)?;
            group.sinks.push(sink);
            Ok(())
        })
    }

    pub fn play_now_with_volume(
        &mut self,
        key: &str,
        bytes: &'static [u8],
        volume: u8,
    ) -> Result<(), PlayNowError> {
        self.with_sink(key, |group, device| {
            let mut sink = device.open_sink()?;
            sink.play_now(bytes)?;
            sink.set_volume(f32::from(volume) / 255.0)?;
            group.sinks.push(sink);
            Ok(())
        })
    }

    fn with_sink<F, T>(&mut self, key: &str, consumer: F) -> T
    where
        F: FnOnce(&mut AudioSinkGroup, &mut dyn AudioDevice) -> T,
    {
        loop {
            if let Some(group) = self.sinks.get_mut(key) {
                break consumer(group, &mut self.device);
            }
            self.sinks.insert(key.to_owned(), AudioSinkGroup::new());
        }
    }
}
