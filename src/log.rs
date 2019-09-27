use std::{fs::OpenOptions, io};

#[derive(Debug, Clone)]
pub struct Crash;

impl io::Write for Crash {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        OpenOptions::new()
            .append(true)
            .create(true)
            .open("thedes_crash.txt")?
            .write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Debug;

#[cfg(debug_assertions)]
impl io::Write for Debug {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        OpenOptions::new()
            .append(true)
            .create(true)
            .open("thedes_debug.txt")?
            .write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(not(debug_assertions))]
impl io::Write for Debug {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
