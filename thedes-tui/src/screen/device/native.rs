use std::fmt::Write as _;

use crossterm::{
    Command as _,
    cursor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use thedes_async_util::dyn_async_trait;
use tokio::io::{self, AsyncWriteExt, Stdout};

use crate::color::native_ext::ColorToCrossterm;

use super::{Command, Error, ScreenDevice};

pub fn open() -> Box<dyn ScreenDevice> {
    Box::new(NativeScreenDevice::new())
}

#[derive(Debug)]
struct NativeScreenDevice {
    buf: String,
    target: Stdout,
}

impl NativeScreenDevice {
    pub fn new() -> Self {
        Self { buf: String::new(), target: io::stdout() }
    }
}

#[dyn_async_trait]
impl ScreenDevice for NativeScreenDevice {
    fn run(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::Enter => {
                EnterAlternateScreen.write_ansi(&mut self.buf)?;
            },

            Command::Leave => {
                LeaveAlternateScreen.write_ansi(&mut self.buf)?;
            },

            Command::Clear => {
                write!(
                    self.buf,
                    "{}",
                    terminal::Clear(terminal::ClearType::All)
                )?;
            },

            Command::ResetBackground => {
                write!(
                    self.buf,
                    "{}",
                    crossterm::style::SetBackgroundColor(
                        crossterm::style::Color::Reset
                    )
                )?;
            },

            Command::ResetForeground => {
                write!(
                    self.buf,
                    "{}",
                    crossterm::style::SetForegroundColor(
                        crossterm::style::Color::Reset
                    )
                )?;
            },

            Command::SetBackground(color) => {
                write!(
                    self.buf,
                    "{}",
                    crossterm::style::SetBackgroundColor(color.to_crossterm()),
                )?;
            },

            Command::SetForeground(color) => {
                write!(
                    self.buf,
                    "{}",
                    crossterm::style::SetForegroundColor(color.to_crossterm()),
                )?;
            },

            Command::ShowCursor => {
                write!(self.buf, "{}", cursor::Show)?;
            },

            Command::HideCursor => {
                write!(self.buf, "{}", cursor::Hide)?;
            },

            Command::MoveCursor(point) => {
                write!(self.buf, "{}", cursor::MoveTo(point.x, point.y))?;
            },

            Command::Write(ch) => {
                self.buf.push(ch);
            },
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Error> {
        if !self.buf.is_empty() {
            self.target.write_all(self.buf.as_bytes()).await?;
            self.buf.clear();
        }
        Ok(())
    }
}
