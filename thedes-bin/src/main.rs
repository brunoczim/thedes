use std::{
    backtrace::Backtrace,
    env,
    error::Error,
    fs,
    io,
    panic,
    path::PathBuf,
    sync::{Arc, atomic},
};

use chrono::{Datelike, Timelike};
use thiserror::Error;
use tokio::runtime;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    EnvFilter,
    Layer,
    filter::FromEnvError,
    layer::SubscriberExt,
    util::{SubscriberInitExt, TryInitError},
};

const LOG_ENABLED_ENV_VAR: &'static str = "THEDES_LOG";
const LOG_LEVEL_ENV_VAR: &'static str = "THEDES_LOG_LEVEL";
const LOG_PATH_ENV_VAR: &'static str = "THEDES_LOG_PATH";

const THREAD_STACK_SIZE: usize = 4 * 1024 * 1024;

#[derive(Debug, Error)]
enum ProgramError {
    #[error("Failed to open log file {}", .path.display())]
    OpenLogFile {
        path: PathBuf,
        #[source]
        cause: io::Error,
    },
    #[error("Failed to get log filter")]
    LogFilter(
        #[source]
        #[from]
        FromEnvError,
    ),
    #[error("Failed to init logger")]
    LogInit(
        #[source]
        #[from]
        TryInitError,
    ),
    #[error("Failed to start asynchronous runtime")]
    AsyncRuntime(#[source] io::Error),
    #[error(transparent)]
    TuiApp(#[from] thedes_app::Error),
    #[error(transparent)]
    TuiRuntime(#[from] thedes_tui::core::runtime::Error),
}

async fn async_runtime_main() -> Result<(), ProgramError> {
    let config = thedes_tui::core::runtime::Config::new();
    let runtime_future = config.run(thedes_app::run);
    runtime_future.await??;
    Ok(())
}

fn setup_logger() -> Result<(), ProgramError> {
    let mut options = fs::OpenOptions::new();

    options.write(true).append(true).create(true).truncate(false);

    let path = match env::var_os(LOG_PATH_ENV_VAR) {
        Some(path) => path.into(),
        None => {
            let now = chrono::Local::now();
            let stem = format!(
                "log_{:04}-{:02}-{:02}_{:02}-{:02}-{:02}.txt",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second(),
            );
            match directories::ProjectDirs::from(
                "io.github",
                "brunoczim",
                "Thedes",
            ) {
                Some(dirs) => dirs.cache_dir().join(stem),
                None => stem.into(),
            }
        },
    };

    if let Some(dir) = path.parent() {
        fs::create_dir_all(&dir).map_err(|cause| {
            ProgramError::OpenLogFile { path: path.clone(), cause }
        })?;
    }

    let file = options
        .open(&path)
        .map_err(|cause| ProgramError::OpenLogFile { path, cause })?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(Arc::new(file))
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::INFO.into())
                        .with_env_var(LOG_LEVEL_ENV_VAR)
                        .from_env()?,
                ),
        )
        .try_init()?;

    Ok(())
}

fn setup_panic_handler() {
    panic::set_hook(Box::new(|info| {
        tracing::error!("{}\n", info);
        let backtrace = Backtrace::capture();
        tracing::error!("backtrace:\n{}\n", backtrace);
        eprintln!("{}", info);
        eprintln!("backtrace:\n{}\n", backtrace);
    }));
}

fn try_main() -> Result<(), ProgramError> {
    setup_panic_handler();

    let mut log_enabled = false;
    if env::var_os(LOG_LEVEL_ENV_VAR).is_some()
        || env::var_os(LOG_PATH_ENV_VAR).is_some()
    {
        log_enabled = true;
    }
    if let Some(log_enabled_var) = env::var_os(LOG_ENABLED_ENV_VAR) {
        if log_enabled_var.eq_ignore_ascii_case("true")
            || log_enabled_var.eq_ignore_ascii_case("t")
            || log_enabled_var.eq_ignore_ascii_case("on")
            || log_enabled_var.eq_ignore_ascii_case("yes")
            || log_enabled_var.eq_ignore_ascii_case("y")
            || log_enabled_var == "1"
        {
            log_enabled = true;
        } else if log_enabled_var.eq_ignore_ascii_case("false")
            || log_enabled_var.eq_ignore_ascii_case("f")
            || log_enabled_var.eq_ignore_ascii_case("off")
            || log_enabled_var.eq_ignore_ascii_case("no")
            || log_enabled_var.eq_ignore_ascii_case("n")
            || log_enabled_var == "0"
        {
            log_enabled = false;
        }
    }

    if log_enabled {
        setup_logger()?;
    }

    let runtime = runtime::Builder::new_multi_thread()
        .enable_time()
        .thread_stack_size(THREAD_STACK_SIZE)
        .build()
        .map_err(ProgramError::AsyncRuntime)?;

    runtime.block_on(async_runtime_main())
}

fn main() {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
        atomic::fence(atomic::Ordering::SeqCst);
    }

    if let Err(error) = try_main() {
        eprintln!("thedes found a fatal error!");
        let mut current_error = Some(&error as &dyn Error);
        while let Some(error) = current_error {
            eprintln!("caused by:");
            eprintln!("  {error}");
            current_error = error.source();
        }
    }
}
