use std::{sync::atomic::Ordering::*, time::Duration};

use device::RuntimeDevice;
use thedes_async_util::{
    non_blocking::spsc::watch::AtomicMessage,
    timer::Timer,
};
use thiserror::Error;
use tokio::task::JoinError;
use tokio_util::sync::CancellationToken;

use crate::{app::App, grapheme, input, screen, status::Status};

pub mod device;

pub(crate) type JoinSet = tokio::task::JoinSet<Result<(), Error>>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to render to the screen")]
    Screen(
        #[from]
        #[source]
        screen::Error,
    ),
    #[error("Failed to process input events")]
    Input(
        #[from]
        #[source]
        input::Error,
    ),
    #[error("Failed to initialize runtime device")]
    DeviceInit(
        #[from]
        #[source]
        device::Error,
    ),
    #[error("Failed to join a task")]
    JoinError(
        #[from]
        #[source]
        JoinError,
    ),
    #[error("App was unexpectedly cancelled")]
    AppCancelled,
}

#[derive(Debug)]
pub struct Config {
    cancel_token: CancellationToken,
    screen: screen::Config,
    input: input::Config,
    tick_period: Duration,
    device: Option<Box<dyn RuntimeDevice>>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            cancel_token: CancellationToken::new(),
            screen: screen::Config::new(),
            input: input::Config::new(),
            tick_period: Duration::from_millis(8),
            device: None,
        }
    }

    pub fn with_cancel_token(self, token: CancellationToken) -> Self {
        Self { cancel_token: token, ..self }
    }

    pub fn with_screen(self, config: screen::Config) -> Self {
        Self { screen: config, ..self }
    }

    pub fn with_input(self, config: input::Config) -> Self {
        Self { input: config, ..self }
    }

    pub fn with_tick_period(self, duration: Duration) -> Self {
        Self { tick_period: duration, ..self }
    }

    pub fn with_device(self, device: Box<dyn RuntimeDevice>) -> Self {
        Self { device: Some(device), ..self }
    }

    pub async fn run<F, A>(self, app_scope: F) -> Result<A::Output, Error>
    where
        F: FnOnce(App) -> A,
        A: Future + Send + 'static,
        A::Output: Send + 'static,
    {
        let mut device = self.device.unwrap_or_else(device::native::open);
        let _panic_restore_guard = device.open_panic_restore_guard();

        let grapheme_registry = grapheme::Registry::new();
        let timer = Timer::new(self.tick_period);
        let status = Status::new();

        let mut join_set = JoinSet::new();

        let input_handles = self.input.open(
            input::OpenResources {
                device: device.open_input_device(),
                status: status.clone(),
            },
            &mut join_set,
        );

        let screen_handles = self.screen.open(
            screen::OpenResources {
                device: device.open_screen_device(),
                grapheme_registry: grapheme_registry.clone(),
                cancel_token: self.cancel_token.clone(),
                timer: timer.clone(),
                term_size_watch: input_handles.term_size,
                status,
            },
            &mut join_set,
        );

        device.blocking_init()?;

        let app = App {
            grapheme_registry,
            tick_session: timer.new_session(),
            events: input_handles.events,
            canvas: screen_handles.canvas,
            cancel_token: self.cancel_token.clone(),
        };
        let app_output = app.run(&mut join_set, app_scope);

        let mut errors = Vec::new();
        while let Some(join_result) = join_set.join_next().await {
            let result = match join_result {
                Ok(Ok(())) => {
                    tracing::trace!("Joined task OK");
                    Ok(())
                },
                Ok(Err(error)) => Err(error),
                Err(error) => Err(error.into()),
            };

            if let Err(error) = result {
                tracing::error!("Failed to join task: {error:#?}");
                self.cancel_token.cancel();
                errors.push(error);
            }
        }

        if let Err(error) = device.blocking_shutdown() {
            errors.push(error.into());
        }

        let mut result = Ok(app_output.take(Relaxed));
        for error in errors {
            if result.is_ok() {
                self.cancel_token.cancel();
            }
            match error {
                Error::JoinError(join_error) if join_error.is_panic() => {
                    std::panic::resume_unwind(join_error.into_panic())
                },
                _ => (),
            }
            if result.is_ok() {
                result = Err(error);
            }
        }
        result.and_then(|maybe_output| maybe_output.ok_or(Error::AppCancelled))
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use thiserror::Error;
    use tokio::{task, time::timeout};
    use tokio_util::sync::CancellationToken;

    use crate::{
        color::{BasicColor, ColorPair},
        event::{Event, InternalEvent, Key, KeyEvent, ResizeEvent},
        geometry::CoordPair,
        mutation::{MutationExt, Set},
        runtime::{Config, device::mock::RuntimeDeviceMock},
        screen::{self, Command},
        tile::{MutateColors, MutateGrapheme},
    };

    #[derive(Debug, Error)]
    enum TestError {
        #[error("Failed to interact with screen canvas")]
        CanvasFlush(
            #[from]
            #[source]
            crate::screen::FlushError,
        ),
    }

    async fn tui_main(mut app: crate::App) -> Result<(), TestError> {
        let colors = ColorPair {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightGreen.into(),
        };
        let message = "Hello, World!";

        'main: loop {
            let mut x = 0;
            for ch in message.chars() {
                app.canvas.queue([Command::new_mutation(
                    CoordPair { x, y: 0 },
                    MutateGrapheme(Set(ch.into()))
                        .then(MutateColors(Set(colors))),
                )]);
                x += 1;
            }
            if app.canvas.flush().is_err() {
                eprintln!("Screen command receiver disconnected");
                break;
            }
            let Ok(events) = app.events.read_until_now() else {
                eprintln!("Event sender disconnected");
                break;
            };
            for event in events {
                match event {
                    Event::Key(KeyEvent {
                        alt: false,
                        ctrl: false,
                        shift: false,
                        main_key: Key::Char('q') | Key::Char('Q') | Key::Esc,
                    }) => break 'main,
                    _ => (),
                }
            }
            tokio::select! {
                _ = app.tick_session.tick() => (),
                _ = app.cancel_token.cancelled() => break,
            }
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn quit_on_q_with_default_canvas_size() {
        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        let device = device_mock.open();
        let config = Config::new()
            .with_screen(screen::Config::new())
            .with_device(device);

        device_mock.input().publish_ok([InternalEvent::External(Event::Key(
            KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key: Key::Char('q'),
            },
        ))]);

        let runtime_future = task::spawn(config.run(tui_main));
        timeout(Duration::from_millis(200), runtime_future)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn quit_on_esc() {
        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        let device = device_mock.open();
        let config = Config::new()
            .with_screen(
                screen::Config::new()
                    .with_canvas_size(CoordPair { y: 22, x: 78 }),
            )
            .with_device(device);

        device_mock.input().publish_ok([InternalEvent::External(Event::Key(
            KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key: Key::Esc,
            },
        ))]);

        let runtime_future = task::spawn(config.run(tui_main));
        timeout(Duration::from_millis(200), runtime_future)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn print_message() {
        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        device_mock.screen().enable_command_log();
        let device = device_mock.open();
        let config = Config::new()
            .with_tick_period(Duration::from_millis(1))
            .with_screen(
                screen::Config::new()
                    .with_canvas_size(CoordPair { y: 22, x: 78 }),
            )
            .with_device(device);

        let runtime_future = task::spawn(config.run(tui_main));

        tokio::time::sleep(Duration::from_millis(10)).await;

        device_mock.input().publish_ok([InternalEvent::External(Event::Key(
            KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key: Key::Char('q'),
            },
        ))]);

        timeout(Duration::from_millis(200), runtime_future)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();

        let command_log = device_mock.screen().take_command_log().unwrap();

        let message = "Hello, World!";
        for ch in message.chars() {
            assert_eq!(
                message.chars().filter(|resize_ch| *resize_ch == ch).count(),
                command_log
                    .iter()
                    .flatten()
                    .filter(|command| **command
                        == screen::device::Command::Write(ch))
                    .count(),
                "expected {ch} to occur once, commands: {command_log:#?}",
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_token_stops() {
        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        let device = device_mock.open();
        let cancel_token = CancellationToken::new();
        let config = Config::new()
            .with_cancel_token(cancel_token.clone())
            .with_screen(
                screen::Config::new()
                    .with_canvas_size(CoordPair { y: 22, x: 78 }),
            )
            .with_device(device);

        let runtime_future = task::spawn(config.run(tui_main));
        cancel_token.cancel();
        timeout(Duration::from_millis(200), runtime_future)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn resize_too_small() {
        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        let device = device_mock.open();
        let config = Config::new()
            .with_screen(
                screen::Config::new()
                    .with_canvas_size(CoordPair { y: 22, x: 78 }),
            )
            .with_device(device);

        device_mock.input().publish_ok([InternalEvent::Resize(ResizeEvent {
            size: CoordPair { y: 23, x: 79 },
        })]);

        let runtime_future = task::spawn(config.run(tui_main));
        tokio::time::sleep(Duration::from_millis(50)).await;

        device_mock.input().publish_ok([InternalEvent::External(Event::Key(
            KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key: Key::Esc,
            },
        ))]);
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!device_mock.panic_restore().called());

        device_mock.input().publish_ok([InternalEvent::Resize(ResizeEvent {
            size: CoordPair { y: 24, x: 80 },
        })]);
        tokio::time::sleep(Duration::from_millis(50)).await;
        device_mock.input().publish_ok([InternalEvent::External(Event::Key(
            KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key: Key::Esc,
            },
        ))]);

        timeout(Duration::from_millis(200), runtime_future)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();

        assert!(device_mock.panic_restore().called());
    }
}
