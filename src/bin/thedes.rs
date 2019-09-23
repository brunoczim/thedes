use backtrace::Backtrace;
use std::{fs::File, io::Write, panic, process};
use thedes::backend::Termion;

fn handle_panic(info: &panic::PanicInfo) {
    let mut file =
        File::create("thedes_crash.txt").expect("Couldn't create log");
    write!(file, "{}\n\n", info).expect("Couldn't write log");

    let backtrace = Backtrace::new();
    write!(file, "{:?}", backtrace).expect("Couldn't write log");
}

fn main() {
    panic::set_hook(Box::new(handle_panic));

    if let Err(e) = thedes::game_main::<Termion>() {
        eprintln!("{}", e);
        process::exit(-1);
    }
}
