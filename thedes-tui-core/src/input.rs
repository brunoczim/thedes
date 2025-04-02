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
    pub events: EventReader,
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
        InputHandles { events: event_reader, term_size: term_size_watch }
    }
}

#[derive(Debug)]
pub struct TermSizeWatch {
    receiver: non_blocking::spsc::watch::Receiver<MessageBox<CoordPair>>,
}

impl TermSizeWatch {
    pub(crate) fn new(
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

            if !self.term_size_sender.is_connected() {
                tracing::info!("Terminal size receiver disconnected");
                break;
            }
            if !self.external_event_sender.is_connected() {
                tracing::info!("External event receiver disconnected");
                break;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::{io, time::timeout};

    use crate::{
        event::{Event, InternalEvent, Key, KeyEvent, ResizeEvent},
        geometry::CoordPair,
        input::{
            InputHandles,
            OpenResources,
            device::{self, mock::InputDeviceMock},
        },
        runtime::JoinSet,
    };

    use super::Config;

    #[tokio::test]
    async fn send_correct_events() {
        let device_mock = InputDeviceMock::new();
        let device = device_mock.open();
        let mut join_set = JoinSet::new();
        let resources = OpenResources { device };

        device_mock.publish_ok([
            InternalEvent::External(Event::Key(KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key: Key::Esc,
            })),
            InternalEvent::Resize(ResizeEvent {
                size: CoordPair { y: 30, x: 100 },
            }),
            InternalEvent::External(Event::Key(KeyEvent {
                alt: false,
                ctrl: true,
                shift: false,
                main_key: Key::Enter,
            })),
        ]);

        let mut handles = Config::new()
            .with_poll_timeout(Duration::from_millis(1))
            .open(resources, &mut join_set);

        let max_tries = 10;
        let mut esc_event = None;
        for _ in 0 .. max_tries {
            esc_event = handles.events.read_one().unwrap();
            if esc_event.is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        let esc_event = esc_event.unwrap();

        assert_eq!(
            esc_event,
            Event::Key(KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key: Key::Esc,
            })
        );

        let enter_event = handles.events.read_one().unwrap().unwrap();
        assert_eq!(
            enter_event,
            Event::Key(KeyEvent {
                alt: false,
                ctrl: true,
                shift: false,
                main_key: Key::Enter,
            })
        );

        assert_eq!(handles.events.read_one().unwrap(), None);

        let resize_event = handles.term_size.recv().unwrap().unwrap();
        assert_eq!(resize_event, CoordPair { y: 30, x: 100 },);

        assert_eq!(handles.term_size.recv().unwrap(), None);

        drop(handles);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }
    }

    #[tokio::test]
    async fn use_configured_timeout() {
        let device_mock = InputDeviceMock::new();
        device_mock.enable_timeout_log();

        let device = device_mock.open();
        let mut join_set = JoinSet::new();
        let resources = OpenResources { device };

        device_mock.publish_ok([
            InternalEvent::External(Event::Key(KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key: Key::Esc,
            })),
            InternalEvent::Resize(ResizeEvent {
                size: CoordPair { y: 30, x: 100 },
            }),
            InternalEvent::External(Event::Key(KeyEvent {
                alt: false,
                ctrl: true,
                shift: false,
                main_key: Key::Enter,
            })),
        ]);

        let handles = Config::new()
            .with_poll_timeout(Duration::from_millis(1))
            .open(resources, &mut join_set);

        drop(handles);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }

        let timeout_log = device_mock.take_timeout_log().unwrap();
        for timeout in timeout_log {
            assert_eq!(timeout, Duration::from_millis(1));
        }
    }

    #[tokio::test]
    async fn stop_on_error() {
        let device_mock = InputDeviceMock::new();
        let device = device_mock.open();
        let mut join_set = JoinSet::new();
        let resources = OpenResources { device };

        let error = io::ErrorKind::Unsupported.into();
        device_mock.publish([
            Err(device::Error::Io(error)),
            Ok(InternalEvent::External(Event::Key(KeyEvent {
                alt: false,
                ctrl: true,
                shift: false,
                main_key: Key::Enter,
            }))),
        ]);

        let handles = Config::new()
            .with_poll_timeout(Duration::from_millis(1))
            .open(resources, &mut join_set);

        drop(handles);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        assert!(
            results.iter().any(|result| result.is_err()),
            "results: {results:#?}",
        );
    }

    #[tokio::test]
    async fn stops_if_event_listener_disconnects() {
        let device_mock = InputDeviceMock::new();
        let device = device_mock.open();
        let mut join_set = JoinSet::new();
        let resources = OpenResources { device };

        device_mock.publish_ok([
            InternalEvent::External(Event::Key(KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key: Key::Esc,
            })),
            InternalEvent::Resize(ResizeEvent {
                size: CoordPair { y: 30, x: 100 },
            }),
            InternalEvent::External(Event::Key(KeyEvent {
                alt: false,
                ctrl: true,
                shift: false,
                main_key: Key::Enter,
            })),
        ]);

        let handles = Config::new()
            .with_poll_timeout(Duration::from_millis(1))
            .open(resources, &mut join_set);

        let InputHandles { events, term_size } = handles;

        drop(events);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }

        drop(term_size);
    }

    #[tokio::test]
    async fn stops_if_term_size_watch_disconnects() {
        let device_mock = InputDeviceMock::new();
        let device = device_mock.open();
        let mut join_set = JoinSet::new();
        let resources = OpenResources { device };

        device_mock.publish_ok([
            InternalEvent::External(Event::Key(KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key: Key::Esc,
            })),
            InternalEvent::Resize(ResizeEvent {
                size: CoordPair { y: 30, x: 100 },
            }),
            InternalEvent::External(Event::Key(KeyEvent {
                alt: false,
                ctrl: true,
                shift: false,
                main_key: Key::Enter,
            })),
        ]);

        let handles = Config::new()
            .with_poll_timeout(Duration::from_millis(1))
            .open(resources, &mut join_set);

        let InputHandles { events, term_size } = handles;

        drop(term_size);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }

        drop(events);
    }
}
