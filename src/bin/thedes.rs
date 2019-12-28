use backtrace::Backtrace;
use std::{fs::OpenOptions, panic, process};
use thedes::{
    error::{exit_on_error, restore_term, GameResult},
    storage,
};
use tokio::{runtime::Runtime, task};
use tracing::subscriber;
use tracing_subscriber::fmt::Subscriber;

fn main() {
    let mut runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Error building runtime: {}", e);
            process::exit(-1);
        },
    };

    runtime.block_on(async {
        let _ = task::spawn(async_main()).await;
    });
}

/// Called by the real main inside the runtime block_on;
async fn async_main() {
    if let Err(e) = setup_logger().await {
        // We're exiting below, so, no problem blocking.
        eprintln!("Error opening logger: {}", e);
        eprintln!("{:?}", e.backtrace());
        process::exit(-1);
    }
    setup_panic_handler();

    exit_on_error(thedes::game_main().await);
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
        task::block_in_place(|| {
            let _ = restore_term();
            eprintln!("{}", info);
        });
        tracing::error!("{}\n", info);
        let backtrace = Backtrace::new();
        tracing::error!("{:?}", backtrace);
    }));
}
