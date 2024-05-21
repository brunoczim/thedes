use std::{io, net::SocketAddr, path::PathBuf, process, sync::Arc};

use anyhow::Context;
use chrono::{DateTime, Datelike, Local, Timelike};
use clap::{Parser, Subcommand};
use libthedes::{client, error::Result, server::Server};
use tokio::{fs, signal::ctrl_c, try_join};
use tokio_util::sync::CancellationToken;
use tracing::Subscriber;
use tracing_subscriber::{
    filter::LevelFilter,
    layer::SubscriberExt,
    registry,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};

#[cfg(not(feature = "instrument"))]
use tracing_subscriber::fmt;

#[cfg(feature = "instrument")]
use {
    tracing::Level,
    tracing_subscriber::filter::filter_fn,
    tracing_subscriber::fmt::format::FmtSpan,
};

const LOG_LEVEL_ENV_VAR: &'static str = "THEDES_LOG_LEVEL";

#[derive(Debug, Clone)]
struct ProjectDirs {
    log_dir: PathBuf,
    instrument_dir: PathBuf,
    save_dir: PathBuf,
}

impl ProjectDirs {
    fn fallback() -> Self {
        Self {
            log_dir: PathBuf::from("logs"),
            instrument_dir: PathBuf::from("instrumentations"),
            save_dir: PathBuf::from("saves"),
        }
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "windows",
        target_arch = "wasm32"
    ))]
    fn get() -> Self {
        directories::ProjectDirs::from("io.github", "brunoczim", "Thedes")
            .map_or(Self::fallback(), |os_dirs| Self {
                log_dir: os_dirs.cache_dir().join("logs"),
                instrument_dir: os_dirs.cache_dir().join("instrumentations"),
                save_dir: os_dirs.data_dir().join("saves"),
            })
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "windows",
        target_arch = "wasm32"
    )))]
    fn get() -> Self {
        Self::fallback()
    }
}

#[cfg(not(feature = "instrument"))]
async fn instrument_layer<S>(
    _prefix: &str,
    _dirs: &ProjectDirs,
    _time: DateTime<Local>,
) -> Result<Option<fmt::Layer<S>>>
where
    S: Subscriber,
    for<'a> S: LookupSpan<'a>,
{
    Ok(None)
}

#[cfg(feature = "instrument")]
async fn instrument_layer<S>(
    prefix: &str,
    dirs: &ProjectDirs,
    time: DateTime<Local>,
) -> Result<Option<impl Layer<S> + Send + 'static>>
where
    S: Subscriber,
    for<'a> S: LookupSpan<'a>,
{
    fs::create_dir_all(&dirs.instrument_dir)
        .await
        .with_context(|| dirs.instrument_dir.display().to_string())?;

    let stem = format!(
        "{}_{:04}-{:02}-{:02}_{:02}-{:02}-{:02}.json",
        prefix,
        time.year(),
        time.month(),
        time.day(),
        time.hour(),
        time.minute(),
        time.second(),
    );

    let instrument_path = dirs.instrument_dir.join(&stem);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .truncate(false)
        .open(&instrument_path)
        .with_context(|| instrument_path.display().to_string())?;

    Ok(Some(
        tracing_subscriber::fmt::layer()
            .json()
            .with_span_list(true)
            .with_writer(Arc::new(file))
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_filter(filter_fn(|metadata| {
                *metadata.level() == Level::INFO && metadata.is_span()
            })),
    ))
}

async fn setup_client_logger() -> Result<()> {
    let dirs = ProjectDirs::get();
    fs::create_dir_all(&dirs.log_dir)
        .await
        .with_context(|| dirs.log_dir.display().to_string())?;

    let now = chrono::Local::now();
    let stem = format!(
        "client_{:04}-{:02}-{:02}_{:02}-{:02}-{:02}.txt",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
    );

    let log_path = dirs.log_dir.join(&stem);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .truncate(false)
        .open(&log_path)
        .with_context(|| log_path.display().to_string())?;

    let registry = registry().with(
        tracing_subscriber::fmt::layer()
            .with_writer(Arc::new(file))
            .with_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .with_env_var(LOG_LEVEL_ENV_VAR)
                    .from_env()?,
            ),
    );

    match instrument_layer("client", &dirs, now).await? {
        Some(layer) => registry.with(layer).init(),
        None => registry.init(),
    }
    Ok(())
}

async fn setup_server_logger() -> Result<()> {
    let registry = registry().with(
        tracing_subscriber::fmt::layer().with_writer(io::stderr).with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var(LOG_LEVEL_ENV_VAR)
                .from_env()?,
        ),
    );

    let now = Local::now();
    let dirs = ProjectDirs::get();
    match instrument_layer("server", &dirs, now).await? {
        Some(layer) => registry.with(layer).init(),
        None => registry.init(),
    }

    Ok(())
}

#[derive(Debug, Clone, Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    Launch,
    Serve {
        #[arg(short = 'a', long = "bind-addr")]
        bind_addr: SocketAddr,
    },
}

async fn try_main(cli: Cli) -> Result<()> {
    match cli.cmd {
        Command::Launch => {
            setup_client_logger().await?;
            client::run().await?
        },
        Command::Serve { bind_addr } => {
            setup_server_logger().await?;
            let cancel_token = CancellationToken::new();
            let ctrl_c_task = {
                let cancel_token = cancel_token.clone();
                async move {
                    ctrl_c().await?;
                    cancel_token.cancel();
                    Result::<_>::Ok(())
                }
            };
            let server_task = async move {
                let server =
                    Server::new(bind_addr, cancel_token.clone()).await?;
                server.run().await?;
                cancel_token.cancel();
                Result::<_>::Ok(())
            };
            try_join!(ctrl_c_task, server_task)?;
        },
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(error) = try_main(cli).await {
        for (i, error) in error.chain().enumerate() {
            if i == 0 {
                eprintln!("error: {}", error);
            } else {
                eprintln!("caused by: {}", error);
            }
        }
        eprintln!();
        eprintln!("stack backtrace:");
        eprintln!("{}", error.backtrace());
        process::exit(1);
    }
}
