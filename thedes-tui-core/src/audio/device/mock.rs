use std::{borrow::Cow, collections::VecDeque, mem, sync::Arc};

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

#[derive(Debug)]
struct State {
    open_sink_results:
        VecDeque<Result<Box<dyn AudioSinkDevice>, OpenSinkError>>,
    open_sink_log: Option<u32>,
    device_open: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            open_sink_results: VecDeque::new(),
            device_open: false,
            open_sink_log: None,
        }
    }

    pub fn register_sink(&mut self) -> AudioSinkDeviceMock {
        let sink_mock = AudioSinkDeviceMock::new();
        self.register_open_sink_results([Ok(sink_mock.open())]);
        sink_mock
    }

    pub fn register_open_sink_results(
        &mut self,
        results: impl IntoIterator<
            Item = Result<Box<dyn AudioSinkDevice>, OpenSinkError>,
        >,
    ) {
        self.open_sink_results.extend(results);
    }

    pub fn enable_open_sink_log(&mut self) {
        self.open_sink_log = Some(0);
    }

    pub fn disable_open_sink_log(&mut self) {
        self.open_sink_log = None;
    }

    pub fn take_open_sink_log(&mut self) -> Option<u32> {
        self.open_sink_log.as_mut().map(mem::take)
    }

    pub fn open_sink(
        &mut self,
    ) -> Result<Box<dyn AudioSinkDevice>, OpenSinkError> {
        if let Some(log) = self.open_sink_log.as_mut() {
            *log += 1;
        }
        self.open_sink_results
            .pop_front()
            .unwrap_or_else(|| Ok(AudioSinkDeviceMock::new().open()))
    }

    pub fn mark_device_open(&mut self) -> bool {
        mem::replace(&mut self.device_open, true)
    }
}

#[derive(Debug)]
struct SinkState {
    play_results: VecDeque<Result<(), PlayNowError>>,
    set_volume_results: VecDeque<Result<(), SetVolumeError>>,
    pause_results: VecDeque<Result<(), PauseSinkError>>,
    resume_results: VecDeque<Result<(), ResumeSinkError>>,
    clear_results: VecDeque<Result<(), ClearSinkError>>,
    is_playing_results: VecDeque<Result<(), CheckPlayStatusError>>,
    play_log: Option<Vec<Cow<'static, [u8]>>>,
    set_volume_log: Option<Vec<f32>>,
    sink_open: bool,
    playing: bool,
}

impl SinkState {
    pub fn new() -> Self {
        Self {
            play_results: VecDeque::new(),
            set_volume_results: VecDeque::new(),
            is_playing_results: VecDeque::new(),
            pause_results: VecDeque::new(),
            resume_results: VecDeque::new(),
            clear_results: VecDeque::new(),
            play_log: None,
            set_volume_log: None,
            sink_open: false,
            playing: false,
        }
    }

    pub fn register_play_results(
        &mut self,
        results: impl IntoIterator<Item = Result<(), PlayNowError>>,
    ) {
        self.play_results.extend(results);
    }

    pub fn register_set_volume_results(
        &mut self,
        results: impl IntoIterator<Item = Result<(), SetVolumeError>>,
    ) {
        self.set_volume_results.extend(results);
    }

    pub fn register_pause_results(
        &mut self,
        results: impl IntoIterator<Item = Result<(), PauseSinkError>>,
    ) {
        self.pause_results.extend(results);
    }

    pub fn register_resume_results(
        &mut self,
        results: impl IntoIterator<Item = Result<(), ResumeSinkError>>,
    ) {
        self.resume_results.extend(results);
    }

    pub fn register_clear_results(
        &mut self,
        results: impl IntoIterator<Item = Result<(), ClearSinkError>>,
    ) {
        self.clear_results.extend(results);
    }

    pub fn play_now(
        &mut self,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), PlayNowError> {
        if let Some(play_log) = self.play_log.as_mut() {
            play_log.push(bytes);
        }
        let result = self.play_results.pop_front().unwrap_or(Ok(()));
        if result.is_ok() {
            self.playing = true;
        }
        result
    }

    pub fn pause(&mut self) -> Result<(), PauseSinkError> {
        self.pause_results.pop_front().unwrap_or(Ok(()))?;
        self.playing = false;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), ResumeSinkError> {
        self.resume_results.pop_front().unwrap_or(Ok(()))?;
        self.playing = true;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), ClearSinkError> {
        self.clear_results.pop_front().unwrap_or(Ok(()))?;
        self.playing = false;
        Ok(())
    }

    pub fn force_pause(&mut self) {
        self.playing = false;
    }

    pub fn force_resume(&mut self) {
        self.playing = true;
    }

    pub fn is_playing(&mut self) -> Result<bool, CheckPlayStatusError> {
        match self.is_playing_results.pop_front() {
            Some(Ok(())) | None => Ok(self.playing),
            Some(Err(e)) => Err(e),
        }
    }

    pub fn set_volume(&mut self, volume: f32) -> Result<(), SetVolumeError> {
        if let Some(set_volume_log) = self.set_volume_log.as_mut() {
            set_volume_log.push(volume);
        }
        self.set_volume_results.pop_front().unwrap_or(Ok(()))
    }

    pub fn enable_play_log(&mut self) {
        if self.play_log.is_none() {
            self.play_log = Some(Vec::new());
        }
    }

    pub fn disable_play_log(&mut self) -> Option<Vec<Cow<'static, [u8]>>> {
        self.play_log.take()
    }

    pub fn take_play_log(&mut self) -> Option<Vec<Cow<'static, [u8]>>> {
        self.play_log.as_mut().map(mem::take)
    }

    pub fn enable_set_volume_log(&mut self) {
        if self.set_volume_log.is_none() {
            self.set_volume_log = Some(Vec::new());
        }
    }

    pub fn disable_set_volume_log(&mut self) -> Option<Vec<f32>> {
        self.set_volume_log.take()
    }

    pub fn take_set_volume_log(&mut self) -> Option<Vec<f32>> {
        self.set_volume_log.as_mut().map(mem::take)
    }

    pub fn mark_sink_open(&mut self) -> bool {
        mem::replace(&mut self.sink_open, true)
    }
}

#[derive(Debug, Clone)]
pub struct AudioDeviceMock {
    state: Arc<std::sync::Mutex<State>>,
}

impl AudioDeviceMock {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { state: Arc::new(std::sync::Mutex::new(State::new())) }
    }

    pub fn open(&self) -> Box<dyn AudioDevice> {
        if self.with_state(State::mark_device_open) {
            panic!("Mocked audio sink was already open for this mock");
        }
        Box::new(MockedAudioDevice::new(self.clone()))
    }

    pub fn open_sink(&self) -> Result<Box<dyn AudioSinkDevice>, OpenSinkError> {
        self.with_state(State::open_sink)
    }

    pub fn register_sink(&self) -> AudioSinkDeviceMock {
        self.with_state(State::register_sink)
    }

    pub fn register_open_sink_results(
        &self,
        results: impl IntoIterator<
            Item = Result<Box<dyn AudioSinkDevice>, OpenSinkError>,
        >,
    ) {
        self.with_state(|state| state.register_open_sink_results(results))
    }

    pub fn enable_open_sink_log(&self) {
        self.with_state(State::enable_open_sink_log)
    }

    pub fn disable_open_sink_log(&self) {
        self.with_state(State::disable_open_sink_log)
    }

    pub fn take_open_sink_log(&self) -> Option<u32> {
        self.with_state(State::take_open_sink_log)
    }

    fn with_state<F, T>(&self, scope: F) -> T
    where
        F: FnOnce(&mut State) -> T,
    {
        let mut state = self.state.lock().expect("poisoned lock");
        scope(&mut state)
    }
}

#[derive(Debug, Clone)]
pub struct AudioSinkDeviceMock {
    state: Arc<std::sync::Mutex<SinkState>>,
}

impl AudioSinkDeviceMock {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { state: Arc::new(std::sync::Mutex::new(SinkState::new())) }
    }

    pub fn open(&self) -> Box<dyn AudioSinkDevice> {
        if self.with_state(SinkState::mark_sink_open) {
            panic!("Mocked audio sink was already open for this mock");
        }
        Box::new(MockedAudioSinkDevice::new(self.clone()))
    }

    pub fn enable_play_log(&self) {
        self.with_state(SinkState::enable_play_log)
    }

    pub fn disable_play_log(&self) -> Option<Vec<Cow<'static, [u8]>>> {
        self.with_state(SinkState::disable_play_log)
    }

    pub fn take_play_log(&self) -> Option<Vec<Cow<'static, [u8]>>> {
        self.with_state(SinkState::take_play_log)
    }

    pub fn enable_set_volume_log(&self) {
        self.with_state(SinkState::enable_set_volume_log)
    }

    pub fn disable_set_volume_log(&self) -> Option<Vec<f32>> {
        self.with_state(SinkState::disable_set_volume_log)
    }

    pub fn take_set_volume_log(&self) -> Option<Vec<f32>> {
        self.with_state(SinkState::take_set_volume_log)
    }

    pub fn force_pause(&self) {
        self.with_state(SinkState::force_pause);
    }

    pub fn force_resume(&self) {
        self.with_state(SinkState::force_resume);
    }

    pub fn is_playing(&self) -> Result<bool, CheckPlayStatusError> {
        self.with_state(|state| state.is_playing())
    }

    pub fn register_play_results(
        &self,
        results: impl IntoIterator<Item = Result<(), PlayNowError>>,
    ) {
        self.with_state(|state| state.register_play_results(results))
    }

    pub fn register_set_volume_results(
        &self,
        results: impl IntoIterator<Item = Result<(), SetVolumeError>>,
    ) {
        self.with_state(|state| state.register_set_volume_results(results))
    }

    pub fn register_pause_results(
        &self,
        results: impl IntoIterator<Item = Result<(), PauseSinkError>>,
    ) {
        self.with_state(|state| state.register_pause_results(results))
    }

    pub fn register_resume_results(
        &self,
        results: impl IntoIterator<Item = Result<(), ResumeSinkError>>,
    ) {
        self.with_state(|state| state.register_resume_results(results))
    }

    pub fn register_clear_results(
        &self,
        results: impl IntoIterator<Item = Result<(), ClearSinkError>>,
    ) {
        self.with_state(|state| state.register_clear_results(results))
    }

    fn play_now(
        &mut self,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), PlayNowError> {
        self.with_state(|state| state.play_now(bytes))
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), SetVolumeError> {
        self.with_state(|state| state.set_volume(volume))
    }

    fn pause(&mut self) -> Result<(), PauseSinkError> {
        self.with_state(SinkState::pause)
    }

    fn resume(&mut self) -> Result<(), ResumeSinkError> {
        self.with_state(SinkState::resume)
    }

    fn clear(&mut self) -> Result<(), ClearSinkError> {
        self.with_state(SinkState::clear)
    }

    fn with_state<F, T>(&self, scope: F) -> T
    where
        F: FnOnce(&mut SinkState) -> T,
    {
        let mut state = self.state.lock().expect("poisoned lock");
        scope(&mut state)
    }
}

#[derive(Debug)]
struct MockedAudioDevice {
    mock: AudioDeviceMock,
}

impl MockedAudioDevice {
    pub fn new(mock: AudioDeviceMock) -> Self {
        Self { mock }
    }
}

impl AudioDevice for MockedAudioDevice {
    fn open_sink(&mut self) -> Result<Box<dyn AudioSinkDevice>, OpenSinkError> {
        self.mock.open_sink()
    }
}

#[derive(Debug)]
struct MockedAudioSinkDevice {
    mock: AudioSinkDeviceMock,
}

impl MockedAudioSinkDevice {
    pub fn new(mock: AudioSinkDeviceMock) -> Self {
        Self { mock }
    }
}

impl AudioSinkDevice for MockedAudioSinkDevice {
    fn play_now(
        &mut self,
        bytes: Cow<'static, [u8]>,
    ) -> Result<(), PlayNowError> {
        self.mock.play_now(bytes)
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), SetVolumeError> {
        self.mock.set_volume(volume)
    }

    fn pause(&mut self) -> Result<(), PauseSinkError> {
        self.mock.pause()
    }

    fn resume(&mut self) -> Result<(), ResumeSinkError> {
        self.mock.resume()
    }

    fn clear(&mut self) -> Result<(), ClearSinkError> {
        self.mock.clear()
    }

    fn is_playing(&self) -> Result<bool, CheckPlayStatusError> {
        self.mock.is_playing()
    }
}
