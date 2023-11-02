use std::{
    env,
    fs::OpenOptions,
    io,
    net::SocketAddr,
    path::PathBuf,
    process,
    sync::Arc,
};

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use libthedes::{client, error::Result, server::Server};
use tokio::{signal::ctrl_c, try_join};
use tokio_util::sync::CancellationToken;
use tracing::{level_filters::STATIC_MAX_LEVEL, subscriber};
use tracing_subscriber::{filter::LevelFilter, FmtSubscriber};

fn get_tracing_max_level() -> Result<LevelFilter> {
    let level = match env::var_os("THEDES_LOG_LEVEL") {
        Some(var) if var.eq_ignore_ascii_case("OFF") => LevelFilter::OFF,
        Some(var) if var.eq_ignore_ascii_case("ERROR") => LevelFilter::ERROR,
        Some(var) if var.eq_ignore_ascii_case("WARN") => LevelFilter::WARN,
        Some(var) if var.eq_ignore_ascii_case("INFO") => LevelFilter::INFO,
        Some(var) if var.eq_ignore_ascii_case("DEBUG") => LevelFilter::DEBUG,
        Some(var) if var.eq_ignore_ascii_case("TRACE") => LevelFilter::TRACE,
        Some(var) => {
            Err(anyhow!("Unknown log level {}", var.to_string_lossy()))?
        },
        None => STATIC_MAX_LEVEL,
    };

    Ok(level)
}

async fn setup_client_logger() -> Result<()> {
    let max_level = get_tracing_max_level()?;
    if max_level == LevelFilter::OFF {
        return Ok(());
    }

    let mut log_path = PathBuf::from("thedes-log.txt");
    let mut counter = 0;
    while tokio::fs::try_exists(&log_path).await? {
        counter += 1;
        log_path = PathBuf::from(format!("thedes-log{}.txt", counter));
    }
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .truncate(false)
        .open(log_path)?;
    let shared_file = Arc::new(file);
    let subscriber = FmtSubscriber::builder()
        .with_writer(move || shared_file.clone())
        .with_max_level(max_level)
        .finish();
    subscriber::set_global_default(subscriber)?;
    Ok(())
}

async fn setup_server_logger() -> Result<()> {
    let max_level = get_tracing_max_level()?;
    let subscriber = FmtSubscriber::builder()
        .with_writer(io::stderr)
        .with_max_level(max_level)
        .finish();
    subscriber::set_global_default(subscriber)?;
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
        eprintln!("{}", error);
        eprintln!("{}", error.backtrace());
        process::exit(1);
    }
}
