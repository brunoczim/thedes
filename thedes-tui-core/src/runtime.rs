use std::{sync::atomic::Ordering::*, time::Duration};

use device::RuntimeDevice;
use thedes_async_util::{
    non_blocking::spsc::watch::AtomicMessage,
    timer::Timer,
};
use thiserror::Error;
use tokio::task::JoinError;
use tokio_util::sync::CancellationToken;

use crate::{app::App, grapheme, input, screen};

pub mod device;

pub type JoinSet = tokio::task::JoinSet<Result<(), Error>>;

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
    screen: screen::Config,
    input: input::Config,
    tick_period: Duration,
    device: Option<Box<dyn RuntimeDevice>>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            screen: screen::Config::new(),
            input: input::Config::new(),
            tick_period: Duration::from_millis(8),
            device: None,
        }
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

        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let timer = Timer::new(self.tick_period);

        let mut join_set = JoinSet::new();

        let input_handles = self.input.open(
            input::OpenResources { device: device.open_input_device() },
            &mut join_set,
        );

        let screen_handles = self.screen.open(
            screen::OpenResources {
                device: device.open_screen_device(),
                grapheme_registry: grapheme_registry.clone(),
                cancel_token: cancel_token.clone(),
                timer: timer.clone(),
                term_size_watch: input_handles.term_size,
            },
            &mut join_set,
        );

        let app = App {
            grapheme_registry,
            timer,
            event_reader: input_handles.event,
            canvas: screen_handles.canvas,
            cancel_token: cancel_token.clone(),
        };
        let app_output = app.run(&mut join_set, app_scope);

        let mut errors = Vec::new();
        while let Some(join_result) = join_set.join_next().await {
            let result = match join_result {
                Ok(Ok(())) => Ok(()),
                Ok(Err(error)) => Err(error),
                Err(error) => Err(error.into()),
            };

            if let Err(error) = result {
                cancel_token.cancel();
                errors.push(error);
            }
        }

        let mut result = Ok(app_output.take(Relaxed));
        for error in errors {
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
