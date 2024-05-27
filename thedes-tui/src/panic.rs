use std::io::{self, Write};

use crossterm::{cursor, style, terminal, Command};

pub fn emergency_restore() {
    let _ = terminal::disable_raw_mode();
    print!("{}", cursor::Show);
    print!("{}", style::SetBackgroundColor(crossterm::style::Color::Reset));
    print!("{}", style::SetForegroundColor(crossterm::style::Color::Reset));
    let mut buf = String::new();
    let _ = terminal::LeaveAlternateScreen.write_ansi(&mut buf);
    println!("{}", buf);
    let _ = io::stdout().flush();
}
