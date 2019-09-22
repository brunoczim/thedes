use std::process;
use thedes::backend::Termion;

fn main() {
    if let Err(e) = thedes::game_main::<Termion>() {
        eprintln!("{}", e);
        process::exit(-1);
    }
}
