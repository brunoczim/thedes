use super::Backend;
use crate::{
    key::Key,
    orient::{Coord, Direc},
    render::Color,
};
use std::{
    convert::TryFrom,
    fmt,
    fs::File,
    io::{self, Write},
};
use termion::{
    color,
    cursor::{self, DetectCursorPos},
    event::Key as TermionKey,
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
    AsyncReader,
};

macro_rules! translate_color {
    ($fmt:expr, $fn:path, $color:expr) => {
        match $color {
            Color::Black => write!($fmt, "{}", $fn(color::Black)),
            Color::White => write!($fmt, "{}", $fn(color::White)),
            Color::Red => write!($fmt, "{}", $fn(color::Red)),
            Color::Green => write!($fmt, "{}", $fn(color::Green)),
            Color::Blue => write!($fmt, "{}", $fn(color::Blue)),
            Color::Magenta => write!($fmt, "{}", $fn(color::Magenta)),
            Color::Yellow => write!($fmt, "{}", $fn(color::Yellow)),
            Color::Cyan => write!($fmt, "{}", $fn(color::Cyan)),
            Color::LightBlack => write!($fmt, "{}", $fn(color::LightBlack)),
            Color::LightWhite => write!($fmt, "{}", $fn(color::LightWhite)),
            Color::LightRed => write!($fmt, "{}", $fn(color::LightRed)),
            Color::LightGreen => write!($fmt, "{}", $fn(color::LightGreen)),
            Color::LightBlue => write!($fmt, "{}", $fn(color::LightBlue)),
            Color::LightMagenta => write!($fmt, "{}", $fn(color::LightMagenta)),
            Color::LightYellow => write!($fmt, "{}", $fn(color::LightYellow)),
            Color::LightCyan => write!($fmt, "{}", $fn(color::LightCyan)),
        }
    };
}

fn translate_key(key: TermionKey) -> Option<Key> {
    use TermionKey::*;

    match key {
        Char(ch) => Some(Key::Char(ch)),
        Left => Some(Key::Left),
        Right => Some(Key::Right),
        Up => Some(Key::Up),
        Down => Some(Key::Down),
        _ => None,
    }
}

/// A backend for termion.
pub struct Termion {
    output: AlternateScreen<RawTerminal<File>>,
    input: Keys<AsyncReader>,
}

impl fmt::Debug for Termion {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Termion {{ output: <OUTPUT>, input: <INPUT> }}")
    }
}

impl Write for Termion {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

impl Backend for Termion {
    fn load() -> io::Result<Self> {
        let screen = termion::get_tty()?.into_raw_mode()?;
        let output = AlternateScreen::from(screen);
        let input = termion::async_stdin().keys();
        let mut this = Self { output, input };
        this.goto(0, 0)?;
        Ok(this)
    }

    fn read_key(&mut self) -> io::Result<Option<Key>> {
        self.input.next().transpose().map(|res| res.and_then(translate_key))
    }

    fn goto(&mut self, x: Coord, y: Coord) -> io::Result<()> {
        let res_x = x.checked_add(1).and_then(|x| u16::try_from(x).ok());
        let res_y = y.checked_add(1).and_then(|y| u16::try_from(y).ok());

        let (x, y) = match (res_x, res_y) {
            (Some(x), Some(y)) => (x, y),
            _ => return Err(io::Error::from(io::ErrorKind::InvalidInput)),
        };

        write!(self, "{}", cursor::Goto(x, y))
    }

    fn move_rel(&mut self, direc: Direc, count: Coord) -> io::Result<()> {
        let count = u16::try_from(count)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        match direc {
            Direc::Up => write!(self, "{}", cursor::Up(count)),
            Direc::Left => write!(self, "{}", cursor::Left(count)),
            Direc::Down => write!(self, "{}", cursor::Down(count)),
            Direc::Right => write!(self, "{}", cursor::Right(count)),
        }
    }

    fn pos(&mut self) -> io::Result<(Coord, Coord)> {
        let (x, y) = self.output.cursor_pos()?;
        match (Coord::try_from(x), Coord::try_from(y)) {
            (Ok(x), Ok(y)) => Ok((x, y)),
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }

    fn setbg(&mut self, color: Color) -> io::Result<()> {
        translate_color!(self, color::Bg, color)
    }

    fn setfg(&mut self, color: Color) -> io::Result<()> {
        translate_color!(self, color::Fg, color)
    }
}
