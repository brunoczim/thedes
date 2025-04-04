use std::{collections::VecDeque, mem, sync::Arc};

use thedes_async_util::dyn_async_trait;

use crate::geometry::CoordPair;

use super::{Command, Error, ScreenDevice};

#[derive(Debug)]
struct State {
    term_size: CoordPair,
    term_size_count: usize,
    term_size_results: VecDeque<Result<(), Error>>,
    send_results: VecDeque<Result<usize, Error>>,
    command_log: Option<Vec<Vec<Command>>>,
    flush_results: VecDeque<Result<(), Error>>,
    flush_count: usize,
    device_open: bool,
}

impl State {
    pub fn new(term_size: CoordPair) -> Self {
        Self {
            term_size,
            term_size_count: 0,
            term_size_results: VecDeque::new(),
            send_results: VecDeque::new(),
            command_log: None,
            flush_results: VecDeque::new(),
            flush_count: 0,
            device_open: false,
        }
    }

    pub fn register_term_size_results(
        &mut self,
        results: impl IntoIterator<Item = Result<(), Error>>,
    ) {
        self.term_size_results.extend(results);
    }

    pub fn blocking_term_size(&mut self) -> Result<CoordPair, Error> {
        self.term_size_count += 1;
        self.term_size_results.pop_front().unwrap_or(Ok(()))?;
        Ok(self.term_size)
    }

    pub fn blocking_get_size_count(&self) -> usize {
        self.term_size_count
    }

    pub fn enable_command_log(&mut self) {
        if self.command_log.is_none() {
            self.command_log = Some(vec![vec![]]);
        }
    }

    pub fn disable_command_log(&mut self) -> Option<Vec<Vec<Command>>> {
        self.command_log.take()
    }

    pub fn take_command_log(&mut self) -> Option<Vec<Vec<Command>>> {
        self.command_log
            .as_mut()
            .map(|flushes| mem::replace(flushes, vec![vec![]]))
    }

    pub fn log_command(&mut self, command: Command) {
        if let Some(log) = &mut self.command_log {
            log.last_mut().unwrap().push(command);
        }
    }

    pub fn register_send_results(
        &mut self,
        results: impl IntoIterator<Item = Result<usize, Error>>,
    ) {
        let mut prev_ok = None;
        for result in results {
            match result {
                Ok(count) => {
                    prev_ok = Some(match prev_ok {
                        Some(prev) => prev + count,
                        None => count,
                    });
                },
                Err(error) => {
                    if let Some(prev_count) =
                        prev_ok.filter(|count| *count != 0)
                    {
                        self.send_results.push_back(Ok(prev_count));
                    }
                    self.send_results.push_back(Err(error));
                },
            }
        }
        if let Some(prev_count) = prev_ok.filter(|count| *count != 0) {
            self.send_results.push_back(Ok(prev_count));
        }
    }

    pub fn send(
        &mut self,
        commands: &mut (dyn Iterator<Item = Command> + Send + Sync),
    ) -> Result<(), Error> {
        for command in commands {
            match self.send_results.pop_front().unwrap_or(Ok(1)) {
                Ok(mut count) => {
                    count -= 1;
                    if count > 0 {
                        self.send_results.push_front(Ok(count));
                    }
                    self.log_command(command);
                },
                Err(error) => Err(error)?,
            }
        }
        Ok(())
    }

    pub fn register_flush_results(
        &mut self,
        results: impl IntoIterator<Item = Result<(), Error>>,
    ) {
        self.flush_results.extend(results);
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.flush_count += 1;
        self.flush_results.pop_front().unwrap_or(Ok(()))?;
        if let Some(log) = &mut self.command_log {
            if log.last().is_some_and(|buf| !buf.is_empty()) {
                log.push(Vec::new());
            }
        }
        Ok(())
    }

    pub fn flush_count(&self) -> usize {
        self.flush_count
    }

    pub fn mark_device_open(&mut self) -> bool {
        mem::replace(&mut self.device_open, true)
    }
}

#[derive(Debug, Clone)]
pub struct ScreenDeviceMock {
    state: Arc<std::sync::Mutex<State>>,
}

impl ScreenDeviceMock {
    pub fn new(term_size: CoordPair) -> Self {
        Self { state: Arc::new(std::sync::Mutex::new(State::new(term_size))) }
    }

    pub fn open(&self) -> Box<dyn ScreenDevice> {
        if self.with_state(|state| state.mark_device_open()) {
            panic!("Mocked screen device was already open for this mock");
        }
        Box::new(MockedScreenDevice::new(self.clone()))
    }

    pub fn enable_command_log(&self) {
        self.with_state(|state| state.enable_command_log())
    }

    pub fn disable_command_log(&self) -> Option<Vec<Vec<Command>>> {
        self.with_state(|state| state.disable_command_log())
    }

    pub fn take_command_log(&self) -> Option<Vec<Vec<Command>>> {
        self.with_state(|state| state.take_command_log())
    }

    pub fn register_term_size_results(
        &self,
        results: impl IntoIterator<Item = Result<(), Error>>,
    ) {
        self.with_state(|state| state.register_term_size_results(results))
    }

    pub fn blocking_get_size_count(&self) -> usize {
        self.with_state(|state| state.blocking_get_size_count())
    }

    pub fn register_send_results(
        &self,
        results: impl IntoIterator<Item = Result<usize, Error>>,
    ) {
        self.with_state(|state| state.register_send_results(results))
    }

    pub fn register_flush_results(
        &self,
        results: impl IntoIterator<Item = Result<(), Error>>,
    ) {
        self.with_state(|state| state.register_flush_results(results))
    }

    pub fn flush_count(&self) -> usize {
        self.with_state(|state| state.flush_count())
    }

    fn send(
        &self,
        commands: &mut (dyn Iterator<Item = Command> + Send + Sync),
    ) -> Result<(), Error> {
        self.with_state(|state| state.send(commands))
    }

    async fn flush(&self) -> Result<(), Error> {
        self.with_state(|state| state.flush())
    }

    pub fn blocking_get_size(&self) -> Result<CoordPair, Error> {
        self.with_state(|state| state.blocking_term_size())
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
struct MockedScreenDevice {
    mock: ScreenDeviceMock,
}

impl MockedScreenDevice {
    pub fn new(mock: ScreenDeviceMock) -> Self {
        Self { mock }
    }
}

#[dyn_async_trait]
impl ScreenDevice for MockedScreenDevice {
    fn send_raw(
        &mut self,
        commands: &mut (dyn Iterator<Item = Command> + Send + Sync),
    ) -> Result<(), Error> {
        self.mock.send(commands)
    }

    async fn flush(&mut self) -> Result<(), Error> {
        self.mock.flush().await
    }

    fn blocking_get_size(&mut self) -> Result<CoordPair, Error> {
        self.mock.blocking_get_size()
    }
}
