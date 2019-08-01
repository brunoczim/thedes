use rpgeng::{
    backend::{Backend, Termion},
    tile::Color,
};
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut backend = Termion::load()?;
    backend.setfg(Color::LightYellow)?;
    backend.setbg(Color::Blue)?;
    write!(backend, "Hello, World!")?;
    backend.wait_key()?;

    Ok(())
}
