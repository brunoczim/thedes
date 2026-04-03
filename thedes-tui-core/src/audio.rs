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
    volume: u8,
}

impl AudioSinkGroup {
    fn new() -> Self {
        Self { sinks: Vec::new(), volume: 127 }
    }

    fn float_volume(&self) -> f32 {
        f32::from(self.volume) / 255.0
    }

    fn set_volume(&mut self, volume: u8) -> Result<(), SetVolumeError> {
        self.volume = volume;
        let volume = self.float_volume();
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
            sink.set_volume(group.float_volume())?;
            group.sinks.push(sink);
            Ok(())
        })
    }

    pub fn volume(&mut self, key: &str) -> u8 {
        self.with_sink(key, |group, _device| group.volume)
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

#[cfg(test)]
mod test {
    use crate::audio::{Config, OpenResources, device::mock::AudioDeviceMock};

    #[test]
    fn set_volume_changes_volume() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let mut controller = Config::new().open(OpenResources { device });
        controller.set_volume("Music", 13).unwrap();
        let volume = controller.volume("Music");
        assert_eq!(volume, 13);
    }

    #[test]
    fn set_volume_changes_correct_group() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let mut controller = Config::new().open(OpenResources { device });
        controller.set_volume("Music", 13).unwrap();
        controller.set_volume("FX", 200).unwrap();
        let volume = controller.volume("Music");
        assert_eq!(volume, 13);
        let volume = controller.volume("FX");
        assert_eq!(volume, 200);

        controller.set_volume("Music", 15).unwrap();
        let volume = controller.volume("Music");
        assert_eq!(volume, 15);
        let volume = controller.volume("FX");
        assert_eq!(volume, 200);
    }

    #[test]
    fn play_now_is_propagated() {
        let device_mock = AudioDeviceMock::new();
        device_mock.enable_open_sink_log();
        let sink_mock = device_mock.register_sink();
        sink_mock.enable_play_log();

        let device = device_mock.open();
        let mut controller = Config::new().open(OpenResources { device });
        controller.set_volume("Music", 13).unwrap();
        controller.play_now("Music", &[1, 2, 3]).unwrap();

        assert_eq!(device_mock.take_open_sink_log(), Some(1));
        assert_eq!(
            sink_mock.take_play_log(),
            Some(vec![&[1_u8, 2, 3] as &[u8]])
        );
    }
}
