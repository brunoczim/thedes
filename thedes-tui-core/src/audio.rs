use std::{borrow::Cow, collections::HashMap, mem};

use thedes_async_util::{
    non_blocking,
    timer::{TickSession, Timer},
};
use thiserror::Error;
use tokio::task;
use tokio_util::sync::CancellationToken;

use crate::{
    audio::device::{AudioDevice, AudioSinkDevice},
    runtime,
};

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

#[derive(Debug, Error)]
pub enum Error {
    #[error("Device failed to play")]
    DevicePlayNow(#[from] device::PlayNowError),
    #[error("Device failed to set volume")]
    DeviceSetVolume(#[from] device::SetVolumeError),
    #[error("Device failed to pause a sink")]
    DevicePause(#[from] device::PauseSinkError),
    #[error("Device failed to resume a sink")]
    DeviceResume(#[from] device::ResumeSinkError),
    #[error("Device failed to clear a sink")]
    DeviceClear(#[from] device::ClearSinkError),
    #[error("Device failed to check play status")]
    DeviceIsPlaying(#[from] device::CheckPlayStatusError),
    #[error("Device failed to open sink")]
    DeviceOpenSink(#[from] device::OpenSinkError),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct FlushError {
    inner: non_blocking::spsc::unbounded::SendError<Vec<Command>>,
}

impl FlushError {
    fn new(
        inner: non_blocking::spsc::unbounded::SendError<Vec<Command>>,
    ) -> Self {
        Self { inner }
    }

    pub fn into_bounced_commands(self) -> Vec<Command> {
        self.inner.into_message()
    }
}

#[derive(Debug)]
pub enum Command {
    PlayOnce(Cow<'static, str>, Cow<'static, [u8]>),
    PlayRepeated(Cow<'static, str>, Cow<'static, [u8]>),
    Pause(Cow<'static, str>),
    Resume(Cow<'static, str>),
    Clear(Cow<'static, str>),
    SetVolume(Cow<'static, str>, u8),
}

impl Command {
    pub fn new_play_once(
        name: impl Into<Cow<'static, str>>,
        bytes: impl Into<Cow<'static, [u8]>>,
    ) -> Self {
        Self::PlayOnce(name.into(), bytes.into())
    }

    pub fn new_play_repeated(
        name: impl Into<Cow<'static, str>>,
        bytes: impl Into<Cow<'static, [u8]>>,
    ) -> Self {
        Self::PlayRepeated(name.into(), bytes.into())
    }

    pub fn new_pause(name: impl Into<Cow<'static, str>>) -> Self {
        Self::Pause(name.into())
    }

    pub fn new_resume(name: impl Into<Cow<'static, str>>) -> Self {
        Self::Resume(name.into())
    }

    pub fn new_clear(name: impl Into<Cow<'static, str>>) -> Self {
        Self::Clear(name.into())
    }

    pub fn new_set_volume(
        name: impl Into<Cow<'static, str>>,
        level: u8,
    ) -> Self {
        Self::SetVolume(name.into(), level)
    }
}

#[derive(Debug)]
pub(crate) struct OpenResources {
    pub device: Box<dyn AudioDevice>,
    pub timer: Timer,
    pub cancel_token: CancellationToken,
}

#[derive(Debug)]
pub(crate) struct AudioHandles {
    pub controller: AudioControllerHandle,
}

#[derive(Debug)]
pub struct Config {
    _private: (),
}

impl Config {
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub(crate) fn open(
        self,
        resources: OpenResources,
        join_set: &mut runtime::JoinSet,
    ) -> AudioHandles {
        let (command_sender, command_receiver) =
            non_blocking::spsc::unbounded::channel();

        join_set.spawn(async move {
            let mut reactor = Reactor::new(resources, command_receiver);
            reactor.run().await
        });

        AudioHandles { controller: AudioControllerHandle::new(command_sender) }
    }
}

#[derive(Debug)]
pub struct AudioControllerHandle {
    command_sender: non_blocking::spsc::unbounded::Sender<Vec<Command>>,
    command_queue: Vec<Command>,
}

impl AudioControllerHandle {
    fn new(
        command_sender: non_blocking::spsc::unbounded::Sender<Vec<Command>>,
    ) -> Self {
        Self { command_sender, command_queue: Vec::new() }
    }

    pub fn is_connected(&self) -> bool {
        self.command_sender.is_connected()
    }

    pub fn queue<I>(&mut self, commands: I)
    where
        I: IntoIterator<Item = Command>,
    {
        self.command_queue.extend(commands);
    }

    pub fn flush(&mut self) -> Result<(), FlushError> {
        let commands = mem::take(&mut self.command_queue);
        self.command_sender.send(commands).map_err(FlushError::new)
    }
}

#[derive(Debug)]
struct RepeatedSink {
    inner: Box<dyn AudioSinkDevice>,
    buf: Cow<'static, [u8]>,
}

#[derive(Debug)]
struct AudioSinkGroup {
    once_sinks: Vec<Option<Box<dyn AudioSinkDevice>>>,
    repeated_sinks: Vec<Option<RepeatedSink>>,
    paused: bool,
    volume: u8,
}

impl AudioSinkGroup {
    fn new() -> Self {
        Self {
            once_sinks: Vec::new(),
            paused: false,
            repeated_sinks: Vec::new(),
            volume: 127,
        }
    }

    fn float_volume(&self) -> f32 {
        f32::from(self.volume) / 255.0
    }

    fn set_volume(&mut self, volume: u8) -> Result<(), Error> {
        self.volume = volume;
        let volume = self.float_volume();
        for sink in self
            .repeated_sinks
            .iter_mut()
            .filter_map(|sink| sink.as_mut())
            .map(|sink| &mut sink.inner)
            .chain(self.once_sinks.iter_mut().filter_map(|sink| sink.as_mut()))
        {
            sink.set_volume(volume)?;
        }
        Ok(())
    }

    fn add_once(
        &mut self,
        device: &mut dyn AudioDevice,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), Error> {
        let mut sink = device.open_sink()?;
        sink.play_now(bytes.clone())?;
        sink.set_volume(self.float_volume())?;
        if let Some(entry) = self
            .once_sinks
            .iter_mut()
            .find(|maybe_device| maybe_device.is_none())
        {
            *entry = Some(sink);
        } else {
            self.once_sinks.push(Some(sink));
        }
        if self.paused {
            self.resume()?;
        }
        Ok(())
    }

    fn add_repeated(
        &mut self,
        device: &mut dyn AudioDevice,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), Error> {
        let mut sink = device.open_sink()?;
        sink.play_now(bytes.clone())?;
        sink.set_volume(self.float_volume())?;
        let repeated = RepeatedSink { inner: sink, buf: bytes };
        if let Some(entry) = self
            .repeated_sinks
            .iter_mut()
            .find(|maybe_device| maybe_device.is_none())
        {
            *entry = Some(repeated);
        } else {
            self.repeated_sinks.push(Some(repeated));
        }
        if self.paused {
            self.resume()?;
        }
        Ok(())
    }

    fn pause(&mut self) -> Result<(), Error> {
        self.paused = true;
        for sink in self
            .repeated_sinks
            .iter_mut()
            .filter_map(|sink| sink.as_mut())
            .map(|sink| &mut sink.inner)
            .chain(self.once_sinks.iter_mut().filter_map(|sink| sink.as_mut()))
        {
            sink.pause()?;
        }
        Ok(())
    }

    fn resume(&mut self) -> Result<(), Error> {
        self.paused = false;
        for sink in self
            .repeated_sinks
            .iter_mut()
            .filter_map(|sink| sink.as_mut())
            .map(|sink| &mut sink.inner)
            .chain(self.once_sinks.iter_mut().filter_map(|sink| sink.as_mut()))
        {
            sink.resume()?;
        }
        Ok(())
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.paused = true;
        for mut sink in self
            .repeated_sinks
            .drain(..)
            .filter_map(|sink| sink)
            .map(|sink| sink.inner)
            .chain(self.once_sinks.drain(..).filter_map(|sink| sink))
        {
            sink.clear()?;
        }
        Ok(())
    }

    fn collect_garbage(&mut self) {
        if self.paused {
            return;
        }

        for maybe_sink in &mut self.once_sinks {
            if !maybe_sink
                .as_ref()
                .is_some_and(|sink| sink.is_playing().is_ok_and(|is| is))
            {
                *maybe_sink = None;
            }
        }
        if let Some(last_some) =
            self.once_sinks.iter().rposition(Option::is_some)
        {
            self.once_sinks.drain(last_some + 1 ..);
        }
    }

    fn revive_repeated(&mut self) -> Result<(), Error> {
        for maybe_sink in &mut self.repeated_sinks {
            if let Some(sink) = maybe_sink {
                if !sink.inner.is_playing()? {
                    sink.inner.play_now(sink.buf.clone())?;
                }
            }
        }
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.once_sinks.is_empty() && self.repeated_sinks.is_empty()
    }
}

#[derive(Debug)]
struct Reactor {
    device: Box<dyn AudioDevice>,
    cancel_token: CancellationToken,
    ticker: TickSession,
    command_receiver: non_blocking::spsc::unbounded::Receiver<Vec<Command>>,
    sinks: HashMap<Cow<'static, str>, AudioSinkGroup>,
    groups_to_be_cleared: Vec<Cow<'static, str>>,
}

impl Reactor {
    pub fn new(
        resources: OpenResources,
        command_receiver: non_blocking::spsc::unbounded::Receiver<Vec<Command>>,
    ) -> Self {
        Self {
            device: resources.device,
            cancel_token: resources.cancel_token,
            ticker: resources.timer.new_session(),
            command_receiver,
            sinks: HashMap::new(),
            groups_to_be_cleared: Vec::new(),
        }
    }

    pub async fn run(&mut self) -> Result<(), runtime::Error> {
        let mut commands = Vec::<Command>::new();

        loop {
            if !self.execute_commands_sent(&mut commands)? {
                break;
            }

            tokio::select! {
                _ = self.ticker.tick() => (),
                _ = self.cancel_token.cancelled() => {
                    tracing::info!("Audio reactor token cancellation detected");
                    break
                },
            }

            self.tick()?;

            tokio::select! {
                _ = self.ticker.tick() => (),
                _ = self.cancel_token.cancelled() => {
                    tracing::info!("Audio reactor token cancellation detected");
                    break
                },
            }
        }

        Ok(())
    }

    fn execute_commands_sent(
        &mut self,
        buf: &mut Vec<Command>,
    ) -> Result<bool, Error> {
        let Ok(command_iterator) = self.command_receiver.recv_many() else {
            tracing::info!("Audio reactor command sender disconnected");
            return Ok(false);
        };
        buf.extend(command_iterator.flatten());
        for command in buf.drain(..) {
            self.execute_command(command)?;
        }
        Ok(true)
    }

    fn execute_command(&mut self, command: Command) -> Result<(), Error> {
        task::block_in_place(|| match command {
            Command::PlayOnce(group, bytes) => self.play_once(group, bytes),
            Command::PlayRepeated(group, bytes) => {
                self.play_repeated(group, bytes)
            },
            Command::Pause(group) => self.pause(group),
            Command::Resume(group) => self.resume(group),
            Command::Clear(group) => self.clear(group),
            Command::SetVolume(group, level) => self.set_volume(group, level),
        })
    }

    fn play_once(
        &mut self,
        group: Cow<'static, str>,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), Error> {
        self.with_sink(&group[..], move |group, device| {
            group.add_once(device, bytes)
        })
    }

    fn play_repeated(
        &mut self,
        group: Cow<'static, str>,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), Error> {
        self.with_sink(&group[..], move |group, device| {
            group.add_repeated(device, bytes)
        })
    }

    fn set_volume(
        &mut self,
        group: Cow<'static, str>,
        volume: u8,
    ) -> Result<(), Error> {
        self.with_sink(&group, move |group, _device| group.set_volume(volume))
    }

    fn pause(&mut self, group: Cow<'static, str>) -> Result<(), Error> {
        self.with_sink(&group[..], move |group, _device| group.pause())
    }

    fn resume(&mut self, group: Cow<'static, str>) -> Result<(), Error> {
        self.with_sink(&group[..], move |group, _device| group.resume())
    }

    fn clear(&mut self, group: Cow<'static, str>) -> Result<(), Error> {
        self.with_sink(&group[..], move |group, _device| group.clear())
    }

    fn clear_all(&mut self) {
        for group in self.sinks.values_mut() {
            _ = group.clear();
        }
    }

    fn tick(&mut self) -> Result<(), Error> {
        self.groups_to_be_cleared.clear();
        task::block_in_place(|| {
            for (key, group) in &mut self.sinks {
                group.collect_garbage();
                if group.is_empty() {
                    self.groups_to_be_cleared.push(key.clone());
                }
                group.revive_repeated()?;
            }
            for key in self.groups_to_be_cleared.drain(..) {
                self.sinks.remove(&key);
            }
            Ok(())
        })
    }

    fn with_sink<F, T>(&mut self, group: &str, consumer: F) -> T
    where
        F: FnOnce(&mut AudioSinkGroup, &mut dyn AudioDevice) -> T,
    {
        loop {
            if let Some(group) = self.sinks.get_mut(group) {
                break consumer(group, &mut self.device);
            }
            self.sinks
                .insert(Cow::from(group.to_owned()), AudioSinkGroup::new());
        }
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        if tokio::runtime::Handle::try_current().is_ok() {
            task::block_in_place(|| self.clear_all())
        } else {
            self.clear_all()
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use thedes_async_util::timer::Timer;
    use tokio_util::sync::CancellationToken;

    use crate::{
        audio::{
            Command,
            Config,
            OpenResources,
            device::mock::AudioDeviceMock,
        },
        runtime::JoinSet,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn play_does_play() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let sink_mock = device_mock.register_sink();
        sink_mock.enable_play_log();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_session = timer.new_session();
        let cancel_token = CancellationToken::new();
        let mut join_set = JoinSet::new();

        let mut handles = Config::new()
            .open(OpenResources { device, timer, cancel_token }, &mut join_set);
        handles.controller.queue([Command::new_play_once("Music", &[1, 2, 3])]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        let log = sink_mock.take_play_log().unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0], &[1_u8, 2, 3] as &[u8]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn set_volume_changes_volume() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let sink_mock = device_mock.register_sink();
        sink_mock.enable_set_volume_log();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_session = timer.new_session();
        let cancel_token = CancellationToken::new();
        let mut join_set = JoinSet::new();

        let mut handles = Config::new()
            .open(OpenResources { device, timer, cancel_token }, &mut join_set);
        handles.controller.queue([
            Command::new_play_once("Music", &[1, 2, 3]),
            Command::new_set_volume("Music", 13),
        ]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        let log = sink_mock.take_set_volume_log().unwrap();
        assert_eq!(log[1], 13.0 / 255.0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn play_repeated_plays_repeatedly() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let sink_mock = device_mock.register_sink();
        sink_mock.enable_play_log();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_session = timer.new_session();
        let cancel_token = CancellationToken::new();
        let mut join_set = JoinSet::new();

        let mut handles = Config::new()
            .open(OpenResources { device, timer, cancel_token }, &mut join_set);
        handles
            .controller
            .queue([Command::new_play_repeated("Music", &[1, 2, 3])]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        sink_mock.force_pause();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        let log = sink_mock.take_play_log().unwrap();
        assert_eq!(log.len(), 2);
        for entry in log {
            assert_eq!(entry, &[1_u8, 2, 3] as &[u8]);
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pause_calls_pause() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let sink_mock = device_mock.register_sink();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_session = timer.new_session();
        let cancel_token = CancellationToken::new();
        let mut join_set = JoinSet::new();

        let mut handles = Config::new()
            .open(OpenResources { device, timer, cancel_token }, &mut join_set);
        handles.controller.queue([Command::new_play_once("Music", &[1, 2, 3])]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        handles.controller.queue([Command::new_pause("Music")]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        assert_eq!(sink_mock.is_playing().unwrap(), false);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn resume_calls_pause() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let sink_mock = device_mock.register_sink();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_session = timer.new_session();
        let cancel_token = CancellationToken::new();
        let mut join_set = JoinSet::new();

        let mut handles = Config::new()
            .open(OpenResources { device, timer, cancel_token }, &mut join_set);
        handles.controller.queue([Command::new_play_once("Music", &[1, 2, 3])]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        handles.controller.queue([Command::new_pause("Music")]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        assert_eq!(sink_mock.is_playing().unwrap(), false);

        handles.controller.queue([Command::new_resume("Music")]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        assert_eq!(sink_mock.is_playing().unwrap(), true);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn clear_calls_clear() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let sink_mock = device_mock.register_sink();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_session = timer.new_session();
        let cancel_token = CancellationToken::new();
        let mut join_set = JoinSet::new();

        let mut handles = Config::new()
            .open(OpenResources { device, timer, cancel_token }, &mut join_set);
        handles.controller.queue([Command::new_play_once("Music", &[1, 2, 3])]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        handles.controller.queue([Command::new_clear("Music")]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        assert_eq!(sink_mock.is_playing().unwrap(), false);

        handles.controller.queue([Command::new_resume("Music")]);

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        assert_eq!(sink_mock.is_playing().unwrap(), false);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn play_repeated_plays_repeatedly_on_correct_group() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let music_sink_mock = device_mock.register_sink();
        music_sink_mock.enable_play_log();
        let fx_sink_mock = device_mock.register_sink();
        fx_sink_mock.enable_play_log();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_session = timer.new_session();
        let cancel_token = CancellationToken::new();
        let mut join_set = JoinSet::new();

        let mut handles = Config::new()
            .open(OpenResources { device, timer, cancel_token }, &mut join_set);
        handles.controller.queue([
            Command::new_play_repeated("Music", &[1, 2, 3]),
            Command::new_play_once("FX", &[5, 2, 3]),
        ]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        music_sink_mock.force_pause();
        fx_sink_mock.force_pause();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        let music_log = music_sink_mock.take_play_log().unwrap();
        assert_eq!(music_log.len(), 2);
        for entry in music_log {
            assert_eq!(entry, &[1_u8, 2, 3] as &[u8]);
        }

        let fx_log = fx_sink_mock.take_play_log().unwrap();
        assert_eq!(fx_log.len(), 1);
        for entry in fx_log {
            assert_eq!(entry, &[5_u8, 2, 3] as &[u8]);
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pause_calls_pause_on_correct_group() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let music_sink_mock = device_mock.register_sink();
        let fx_sink_mock = device_mock.register_sink();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_session = timer.new_session();
        let cancel_token = CancellationToken::new();
        let mut join_set = JoinSet::new();

        let mut handles = Config::new()
            .open(OpenResources { device, timer, cancel_token }, &mut join_set);
        handles.controller.queue([
            Command::new_play_once("Music", &[1, 2, 3]),
            Command::new_play_once("FX", &[4, 5, 6]),
        ]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        handles.controller.queue([Command::new_pause("Music")]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        assert_eq!(music_sink_mock.is_playing().unwrap(), false);
        assert_eq!(fx_sink_mock.is_playing().unwrap(), true);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn resume_calls_resume_on_correct_group() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let music_sink_mock = device_mock.register_sink();
        let fx_sink_mock = device_mock.register_sink();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_session = timer.new_session();
        let cancel_token = CancellationToken::new();
        let mut join_set = JoinSet::new();

        let mut handles = Config::new()
            .open(OpenResources { device, timer, cancel_token }, &mut join_set);
        handles.controller.queue([
            Command::new_play_once("Music", &[1, 2, 3]),
            Command::new_play_once("FX", &[4, 5, 6]),
        ]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        handles
            .controller
            .queue([Command::new_pause("Music"), Command::new_pause("FX")]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        handles.controller.queue([Command::new_resume("Music")]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        assert_eq!(music_sink_mock.is_playing().unwrap(), true);
        assert_eq!(fx_sink_mock.is_playing().unwrap(), false);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn clear_calls_clear_on_correct_group() {
        let device_mock = AudioDeviceMock::new();
        let device = device_mock.open();
        let music_sink_mock = device_mock.register_sink();
        let fx_sink_mock = device_mock.register_sink();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_session = timer.new_session();
        let cancel_token = CancellationToken::new();
        let mut join_set = JoinSet::new();

        let mut handles = Config::new()
            .open(OpenResources { device, timer, cancel_token }, &mut join_set);
        handles.controller.queue([
            Command::new_play_once("Music", &[1, 2, 3]),
            Command::new_play_once("FX", &[4, 5, 6]),
        ]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        handles.controller.queue([Command::new_clear("Music")]);
        handles.controller.flush().unwrap();

        tick_session.tick().await;
        tick_session.tick().await;
        tick_session.tick().await;

        assert_eq!(music_sink_mock.is_playing().unwrap(), false);
        assert_eq!(fx_sink_mock.is_playing().unwrap(), true);
    }
}
