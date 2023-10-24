use std::{net::SocketAddr, process};

use clap::{Parser, Subcommand};
use libthedes::{client, error::Result, server::Server};
use tokio_util::sync::CancellationToken;

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
        Command::Launch => client::run().await?,
        Command::Serve { bind_addr } => {
            let server =
                Server::new(bind_addr, CancellationToken::new()).await?;
            server.run().await?;
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
