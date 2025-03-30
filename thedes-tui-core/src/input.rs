use std::time::Duration;

use device::InputDevice;
use thedes_async_util::non_blocking::{self, spsc::watch::MessageBox};
use thiserror::Error;

use crate::{
    event::{Event, InternalEvent},
    geometry::CoordPair,
    runtime,
};

pub mod device;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read event from input device")]
    Device(
        #[from]
        #[source]
        device::Error,
    ),
}

#[derive(Debug, Error)]
#[error("Failed to read value from event reactor")]
pub struct ReadError {
    #[source]
    inner: non_blocking::spsc::bounded::RecvError,
}

impl ReadError {
    fn new(inner: non_blocking::spsc::bounded::RecvError) -> Self {
        Self { inner }
    }
}

#[derive(Debug, Error)]
#[error("Terminal size watch publisher disconnected")]
pub struct TermSizeWatchError {
    #[source]
    inner: non_blocking::spsc::watch::RecvError,
}

impl TermSizeWatchError {
    fn new(inner: non_blocking::spsc::watch::RecvError) -> Self {
        Self { inner }
    }
}

#[derive(Debug)]
pub(crate) struct InputHandles {
    pub event: EventReader,
    pub term_size: TermSizeWatch,
}

#[derive(Debug)]
pub(crate) struct OpenResources {
    pub device: Box<dyn InputDevice>,
}

#[derive(Debug, Clone)]
pub struct Config {
    poll_timeout: Duration,
    buf_size: usize,
}

impl Config {
    pub fn new() -> Self {
        Self { poll_timeout: Duration::from_millis(160), buf_size: 16 }
    }

    pub fn with_poll_timeout(self, duration: Duration) -> Self {
        Self { poll_timeout: duration, ..self }
    }

    pub fn with_buf_size(self, buf_size: usize) -> Self {
        Self { buf_size, ..self }
    }

    pub(crate) fn open(
        self,
        resources: OpenResources,
        join_set: &mut runtime::JoinSet,
    ) -> InputHandles {
        let (event_sender, event_receiver) =
            non_blocking::spsc::bounded::channel(self.buf_size);
        let (term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();

        let mut reactor =
            Reactor::new(self, resources, term_size_sender, event_sender);
        join_set.spawn_blocking(move || reactor.run());

        let event_reader = EventReader::new(event_receiver);
        let term_size_watch = TermSizeWatch::new(term_size_receiver);
        InputHandles { event: event_reader, term_size: term_size_watch }
    }
}

#[derive(Debug)]
pub struct TermSizeWatch {
    receiver: non_blocking::spsc::watch::Receiver<MessageBox<CoordPair>>,
}

impl TermSizeWatch {
    fn new(
        receiver: non_blocking::spsc::watch::Receiver<MessageBox<CoordPair>>,
    ) -> Self {
        Self { receiver }
    }

    pub fn is_connected(&self) -> bool {
        self.receiver.is_connected()
    }

    pub fn recv(&mut self) -> Result<Option<CoordPair>, TermSizeWatchError> {
        self.receiver.recv().map_err(TermSizeWatchError::new)
    }
}

#[derive(Debug)]
pub struct EventReader {
    receiver: non_blocking::spsc::bounded::Receiver<Event>,
}

impl EventReader {
    fn new(receiver: non_blocking::spsc::bounded::Receiver<Event>) -> Self {
        Self { receiver }
    }

    pub fn is_connected(&self) -> bool {
        self.receiver.is_connected()
    }

    pub fn read_one(&mut self) -> Result<Option<Event>, ReadError> {
        self.receiver.recv_one().map_err(ReadError::new)
    }

    pub fn read_until_now<'a>(
        &'a mut self,
    ) -> Result<ReadUntilThen<'a>, ReadError> {
        self.receiver
            .recv_many()
            .map(ReadUntilThen::new)
            .map_err(ReadError::new)
    }
}

#[derive(Debug)]
pub struct ReadUntilThen<'a> {
    inner: non_blocking::spsc::bounded::RecvMany<'a, Event>,
}

impl<'a> ReadUntilThen<'a> {
    fn new(inner: non_blocking::spsc::bounded::RecvMany<'a, Event>) -> Self {
        Self { inner }
    }
}

impl<'a> Iterator for ReadUntilThen<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[derive(Debug)]
struct Reactor {
    device: Box<dyn InputDevice>,
    poll_timeout: Duration,
    term_size_sender: non_blocking::spsc::watch::Sender<MessageBox<CoordPair>>,
    external_event_sender: non_blocking::spsc::bounded::Sender<Event>,
}

impl Reactor {
    fn new(
        config: Config,
        resources: OpenResources,
        resize_sender: non_blocking::spsc::watch::Sender<MessageBox<CoordPair>>,
        external_event_sender: non_blocking::spsc::bounded::Sender<Event>,
    ) -> Self {
        Self {
            device: resources.device,
            poll_timeout: config.poll_timeout,
            term_size_sender: resize_sender,
            external_event_sender,
        }
    }

    pub fn run(&mut self) -> Result<(), runtime::Error> {
        loop {
            if let Some(event) = self
                .device
                .blocking_read(self.poll_timeout)
                .map_err(Error::Device)?
            {
                match event {
                    InternalEvent::Resize(event) => {
                        let _ = self.term_size_sender.send(event.size);
                    },
                    InternalEvent::External(event) => {
                        let _ = self.external_event_sender.send(event);
                    },
                }
            }

            if !self.term_size_sender.is_connected()
                || !self.external_event_sender.is_connected()
            {
                break;
            }
        }
        Ok(())
    }
}
