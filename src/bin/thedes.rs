use backtrace::Backtrace;
use std::{
    fs::OpenOptions,
    io::{self, Write},
    panic,
};
use thedes::{error::GameResult, storage};
use tokio::task;
use tracing::subscriber;
use tracing_subscriber::fmt::Subscriber;

#[tokio::main]
async fn main() {
    if let Err(e) = setup_logger().await {
        task::block_in_place(|| {
            write!(io::stderr(), "Error opening logger: {}", e)
                .expect("Stderr failed?");
        });
    }
    setup_panic_handler();

    /*
    if let Err(e) = thedes::game_main::<DefaultBackend>() {
        eprintln!("{}", e);
        warn!("{}", e);
        warn!("{:?}", e.backtrace());
        process::exit(-1);
    }
    */
}

/// Sets the default logger implementation.
async fn setup_logger() -> GameResult<String> {
    let (name, path) = storage::log_path()?;
    let parent = path.parent().ok_or_else(|| storage::PathAccessError)?;
    storage::ensure_dir(parent).await?;
    let subs = Subscriber::builder()
        .with_writer(move || {
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .expect("error opening log")
        })
        .finish();
    subscriber::set_global_default(subs)?;
    Ok(name)
}

/// Sets the panic handler.
fn setup_panic_handler() {
    panic::set_hook(Box::new(|info| {
        tracing::error!("{}\n\n", info);
        let backtrace = Backtrace::new();
        tracing::error!("{:?}", backtrace);
    }));
}
