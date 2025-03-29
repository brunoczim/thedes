use std::{
    io::{self, Write},
    thread,
};

use crossterm::{Command, cursor, style, terminal};

use super::PanicRestoreGuard;

pub fn open() -> Box<dyn PanicRestoreGuard> {
    Box::new(NativePanicRestoreGuard::new())
}

#[derive(Debug)]
struct NativePanicRestoreGuard {
    enabled: bool,
}

impl NativePanicRestoreGuard {
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

impl PanicRestoreGuard for NativePanicRestoreGuard {
    fn cancel(mut self: Box<Self>) {
        self.enabled = false;
    }
}

impl Drop for NativePanicRestoreGuard {
    fn drop(&mut self) {
        if self.enabled && thread::panicking() {
            let _ = terminal::disable_raw_mode();
            print!("{}", cursor::Show);
            print!(
                "{}",
                style::SetBackgroundColor(crossterm::style::Color::Reset)
            );
            print!(
                "{}",
                style::SetForegroundColor(crossterm::style::Color::Reset)
            );
            let mut buf = String::new();
            let _ = terminal::LeaveAlternateScreen.write_ansi(&mut buf);
            println!("{}", buf);
            let _ = io::stdout().flush();
        }
    }
}
