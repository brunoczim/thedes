use std::io::{self, Write};
use thedes::{
    backend::{Backend, Termion},
    orient::Point,
};

fn main() -> io::Result<()> {
    let mut backend = Termion::load()?;

    write!(backend, "ɾ̩")?;

    backend.goto(Point { x: 1, y: 0 })?;

    write!(backend, "a")?;

    backend.wait_key()?;

    Ok(())
}
