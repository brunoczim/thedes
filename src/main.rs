use std::{io, net::SocketAddr, path::PathBuf, process, sync::Arc};

use anyhow::Context;
use chrono::{Datelike, Timelike};
use clap::{Parser, Subcommand};
use libthedes::{client, error::Result, server::Server};
use tokio::{fs, signal::ctrl_c, try_join};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{
    filter::LevelFilter,
    layer::SubscriberExt,
    registry,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
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
    let log_path = dirs.log_dir.join(stem);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .truncate(false)
        .open(&log_path)
        .with_context(|| log_path.display().to_string())?;

    registry()
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
        .init();
    Ok(())
}

async fn setup_server_logger() -> Result<()> {
    registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(io::stderr)
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::INFO.into())
                        .with_env_var(LOG_LEVEL_ENV_VAR)
                        .from_env()?,
                ),
        )
        .init();
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
