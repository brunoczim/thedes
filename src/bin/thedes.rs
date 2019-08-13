use std::io::{self, Write};
use thedes::backend::{Backend, Termion};

fn main() -> io::Result<()> {
    let mut backend = Termion::load()?;

    write!(backend, "ɾ̩")?;

    backend.goto(1, 0)?;

    write!(backend, "a")?;

    backend.wait_key()?;

    Ok(())
}
