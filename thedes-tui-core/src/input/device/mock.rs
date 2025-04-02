use std::{collections::VecDeque, mem, sync::Arc, time::Duration};

use crate::event::InternalEvent;

use super::{Error, InputDevice};

#[derive(Debug)]
struct State {
    events: VecDeque<Result<InternalEvent, Error>>,
    timeout_log: Option<Vec<Duration>>,
    device_open: bool,
}

impl State {
    pub fn new() -> Self {
        Self { events: VecDeque::new(), timeout_log: None, device_open: false }
    }

    pub fn enable_timeout_log(&mut self) {
        if self.timeout_log.is_none() {
            self.timeout_log = Some(Vec::new());
        }
    }

    pub fn disable_timeout_log(&mut self) -> Option<Vec<Duration>> {
        self.timeout_log.take()
    }

    pub fn take_timeout_log(&mut self) -> Option<Vec<Duration>> {
        self.timeout_log.as_mut().map(mem::take)
    }

    pub fn log_timeout(&mut self, duration: Duration) {
        if let Some(log) = &mut self.timeout_log {
            log.push(duration);
        }
    }

    pub fn publish(
        &mut self,
        events: impl IntoIterator<Item = Result<InternalEvent, Error>>,
    ) {
        self.events.extend(events);
    }

    pub fn read_one(&mut self) -> Result<Option<InternalEvent>, Error> {
        self.events.pop_front().transpose()
    }

    pub fn mark_device_open(&mut self) -> bool {
        mem::replace(&mut self.device_open, true)
    }
}

#[derive(Debug, Clone)]
pub struct InputDeviceMock {
    state: Arc<std::sync::Mutex<State>>,
}

impl InputDeviceMock {
    pub fn new() -> Self {
        Self { state: Arc::new(std::sync::Mutex::new(State::new())) }
    }

    pub fn open(&self) -> Box<dyn InputDevice> {
        if self.with_state(|state| state.mark_device_open()) {
            panic!("Mocked input device was already open for this mock");
        }
        Box::new(MockedInputDevice::new(self.clone()))
    }

    pub fn enable_timeout_log(&self) {
        self.with_state(|state| state.enable_timeout_log())
    }

    pub fn disable_timeout_log(&self) -> Option<Vec<Duration>> {
        self.with_state(|state| state.disable_timeout_log())
    }

    pub fn take_timeout_log(&self) -> Option<Vec<Duration>> {
        self.with_state(|state| state.take_timeout_log())
    }

    pub fn publish(
        &self,
        events: impl IntoIterator<Item = Result<InternalEvent, Error>>,
    ) {
        self.with_state(|state| state.publish(events))
    }

    pub fn publish_ok(&self, events: impl IntoIterator<Item = InternalEvent>) {
        self.publish(events.into_iter().map(Ok));
    }

    pub fn publish_err(&self, events: impl IntoIterator<Item = Error>) {
        self.publish(events.into_iter().map(Err));
    }

    fn read_one(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<InternalEvent>, Error> {
        self.with_state(|state| {
            state.log_timeout(timeout);
            state.read_one()
        })
    }

    fn with_state<F, T>(&self, scope: F) -> T
    where
        F: FnOnce(&mut State) -> T,
    {
        let mut state = self.state.lock().expect("poisoned lock");
        scope(&mut state)
    }
}

#[derive(Debug)]
struct MockedInputDevice {
    mock: InputDeviceMock,
}

impl MockedInputDevice {
    pub fn new(mock: InputDeviceMock) -> Self {
        Self { mock }
    }
}

impl InputDevice for MockedInputDevice {
    fn blocking_read(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<Option<InternalEvent>, Error> {
        self.mock.read_one(timeout)
    }
}
