use backtrace::Backtrace;
use chrono::Local;
use log::{error, warn, Level, LevelFilter, Log, Metadata, Record};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    panic, process,
    sync::Mutex,
};
use thedes::backend::Termion;

fn main() {
    setup_logger();
    setup_panic_handler();

    if let Err(e) = thedes::game_main::<Termion>() {
        eprintln!("{}", e);
        warn!("{}", e);
        process::exit(-1);
    }
}

fn setup_logger() {
    let path = "thedes-log.txt";
    let result = OpenOptions::new().append(true).create(true).open(path);
    if let Ok(file) = result {
        let logger = Logger { file: Mutex::new(file) };
        let ptr = Box::new(logger);
        let _ = log::set_boxed_logger(ptr);
        log::set_max_level(LevelFilter::Trace);
    }
}

fn setup_panic_handler() {
    panic::set_hook(Box::new(|info| {
        error!("{}\n\n", info);
        let backtrace = Backtrace::new();
        error!("{:?}", backtrace);
    }));
}

#[derive(Debug)]
struct Logger {
    file: Mutex<File>,
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let now = Local::now();
        let mut file = self.file.lock().unwrap();

        let _ = write!(
            file,
            "======== {} [{}] =========\n{}\n",
            match record.level() {
                Level::Error => "ERROR",
                Level::Warn => "WARNING",
                Level::Info => "INFO",
                Level::Debug => "DEBUG",
                Level::Trace => "DEBUG",
            },
            now,
            record.args(),
        );
    }

    fn flush(&self) {
        let _ = self.file.lock().unwrap().flush();
    }
}
