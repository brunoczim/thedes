use std::{
    io::{self, Write},
    thread,
};

use crossterm::{Command, cursor, style, terminal};

use crate::{
    input::{self, device::InputDevice},
    panic,
    screen::{self, device::ScreenDevice},
};

use super::{Error, PanicRestoreGuard, RuntimeDevice};

pub fn open() -> Box<dyn RuntimeDevice> {
    Box::new(NativeRuntimeDevice::new())
}

#[derive(Debug)]
struct NativeRuntimeDevice {
    initialized: bool,
}

impl NativeRuntimeDevice {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl RuntimeDevice for NativeRuntimeDevice {
    fn init(&mut self) -> Result<(), Error> {
        if self.initialized {
            Err(Error::AlreadyInit)?
        }
        terminal::enable_raw_mode()?;
        self.initialized = true;
        Ok(())
    }

    fn open_input_device(&mut self) -> Box<dyn InputDevice> {
        input::device::native::open()
    }

    fn open_screen_device(&mut self) -> Box<dyn ScreenDevice> {
        screen::device::native::open()
    }

    fn open_panic_restore_guard(&mut self) -> Box<dyn PanicRestoreGuard> {
        panic::restore::native::open()
    }
}
