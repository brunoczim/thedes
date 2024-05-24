use std::{env, error::Error, fs, io, path::PathBuf, sync::Arc};

use chrono::{Datelike, Timelike};
use thiserror::Error;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    filter::FromEnvError,
    fmt,
    layer::SubscriberExt,
    registry,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};

#[derive(Debug, Error)]
enum ProgramError {
    #[error("Failed to open log file {}", .path.display())]
    OpenLogFile {
        path: PathBuf,
        #[source]
        cause: io::Error,
    },
    #[error("failed to get log filter")]
    LogFilter(
        #[source]
        #[from]
        FromEnvError,
    ),
}

const LOG_ENABLED_ENV_VAR: &'static str = "THEDES_LOG";
const LOG_LEVEL_ENV_VAR: &'static str = "THEDES_LOG_LEVEL";
const LOG_PATH_ENV_VAR: &'static str = "THEDES_LOG_PATH";

fn setup_logger() -> Result<(), ProgramError> {
    let mut options = fs::OpenOptions::new();

    options.write(true).append(true).create(true).truncate(false);

    let file = if let Some(path) = env::var_os(LOG_PATH_ENV_VAR) {
        options.open(&path).map_err(|cause| ProgramError::OpenLogFile {
            path: path.into(),
            cause,
        })?
    } else {
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
        let path = match directories::ProjectDirs::from(
            "io.github",
            "brunoczim",
            "Thedes",
        ) {
            Some(dirs) => dirs.cache_dir().join(stem),
            None => stem.into(),
        };
        options
            .open(&path)
            .map_err(|cause| ProgramError::OpenLogFile { path, cause })?
    };

    registry()
        .with(
            fmt::layer().with_writer(Arc::new(file)).with_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::OFF.into())
                    .with_env_var(LOG_LEVEL_ENV_VAR)
                    .from_env()?,
            ),
        )
        .init();

    Ok(())
}

fn try_main() -> Result<(), ProgramError> {
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

    Ok(())
}

fn main() {
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
