use std::{fmt, io};

use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    Command,
};

#[derive(Debug)]
pub struct TtyScreenDevice<W> {
    writer: W,
}

impl<W> TtyScreenDevice<W>
where
    W: io::Write,
{
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn as_fmt<'a, F>(&'a mut self, operation: F) -> io::Result<()>
    where
        for<'b> F: FnOnce(&'b mut FmtAdapter<&'a mut W>) -> fmt::Result,
    {
        let mut adapter = FmtAdapter::new(&mut self.writer);
        let op_result = operation(&mut adapter);
        adapter.result?;
        if let Err(error) = op_result {
            Err(io::Error::new(io::ErrorKind::Other, error))?;
        }
        Ok(())
    }

    pub fn enter(&mut self) -> io::Result<()> {
        self.as_fmt(|f| EnterAlternateScreen.write_ansi(f))
    }

    pub fn leave(&mut self) -> io::Result<()> {
        self.as_fmt(|f| LeaveAlternateScreen.write_ansi(f))
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[derive(Debug)]
pub struct FmtAdapter<W> {
    result: io::Result<()>,
    writer: W,
}

impl<W> FmtAdapter<W>
where
    W: io::Write,
{
    fn new(writer: W) -> Self {
        Self { result: Ok(()), writer }
    }
}

impl<W> fmt::Write for FmtAdapter<W>
where
    W: io::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.result.is_ok() {
            self.result = self.writer.write_all(s.as_bytes());
        }
        if self.result.is_err() {
            Err(fmt::Error)?;
        }
        Ok(())
    }
}
