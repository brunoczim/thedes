#![deny(unused_must_use)]

/*
use andiskaz::{emergency_restore, terminal};
use backtrace::Backtrace;
use gardiz::coord::Vec2;
use std::{fs::OpenOptions, panic, process, time::Duration};
use thedes::{
    error::{exit_on_error, Result},
    storage,
};
use tokio::{runtime::Runtime, task};
use tracing::{subscriber, Level};
use tracing_subscriber::fmt::{format::FmtSpan, Subscriber};

fn main() {
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("Error building runtime: {}", err);
            process::exit(-1);
        },
    };

    let res = runtime.block_on(async {
        let res = task::spawn(async_main()).await;
        res
    });

    if let Err(err) = res {
        eprintln!("Error setting runtime execution: {}", err);
        process::exit(-1);
    }
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

    let builder = setup_terminal();
    let result = builder.run(thedes::game_main).await;
    exit_on_error(result.map_err(Into::into).and_then(|res| res));
}

/// Sets the default logger implementation.
async fn setup_logger() -> Result<String> {
    let (name, path) = storage::log_path()?;
    let parent = path.parent().ok_or_else(|| storage::PathAccessError)?;
    storage::ensure_dir(parent).await?;
    let level = tracing::level_filters::STATIC_MAX_LEVEL
        .clone()
        .into_level()
        .unwrap_or(Level::DEBUG);
    let subs = Subscriber::builder()
        .with_writer(move || {
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .expect("error opening log")
        })
        .with_max_level(level)
        .with_span_events(FmtSpan::FULL)
        .finish();
    subscriber::set_global_default(subs)?;
    Ok(name)
}

/// Sets the panic handler.
fn setup_panic_handler() {
    panic::set_hook(Box::new(|info| {
        task::block_in_place(|| {
            let _ = emergency_restore();
            eprintln!("{}", info);
        });
        tracing::error!("{}\n", info);
        let backtrace = Backtrace::new();
        tracing::error!("{:?}", backtrace);
    }));
}

/// Prepares a terminal builder.
fn setup_terminal() -> terminal::Builder {
    terminal::Builder::new()
        .frame_time(Duration::from_millis(20))
        .min_screen(Vec2 { x: 80, y: 25 })
}
*/

fn main() {}
