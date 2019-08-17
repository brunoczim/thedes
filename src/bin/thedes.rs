use std::io::{self, Write};
use thedes::{
    backend::{Backend, Termion},
    orient::Coord2D,
};

fn main() -> io::Result<()> {
    let mut backend = Termion::load()?;

    write!(backend, "ɾ̩")?;

    backend.goto(Coord2D::ORIGIN + Coord2D { x: 1, y: 0 })?;

    write!(backend, "a")?;

    backend.wait_key()?;

    Ok(())
}
