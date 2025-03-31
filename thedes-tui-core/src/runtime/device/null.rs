use crate::{
    input::{self, device::InputDevice},
    panic::{self, restore::PanicRestoreGuard},
    screen::{self, device::ScreenDevice},
};

use super::{Error, RuntimeDevice};

pub fn open() -> Box<dyn RuntimeDevice> {
    Box::new(NullRuntimeDevice::new())
}

#[derive(Debug)]
struct NullRuntimeDevice {
    initialized: bool,
}

impl NullRuntimeDevice {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl RuntimeDevice for NullRuntimeDevice {
    fn blocking_init(&mut self) -> Result<(), Error> {
        if self.initialized {
            Err(Error::AlreadyInit)?
        }
        self.initialized = true;
        Ok(())
    }

    fn blocking_shutdown(&mut self) -> Result<(), Error> {
        if !self.initialized {
            Err(Error::NotInit)?
        }
        self.initialized = false;
        Ok(())
    }

    fn open_input_device(&mut self) -> Box<dyn InputDevice> {
        input::device::null::open()
    }

    fn open_screen_device(&mut self) -> Box<dyn ScreenDevice> {
        screen::device::null::open()
    }

    fn open_panic_restore_guard(&mut self) -> Box<dyn PanicRestoreGuard> {
        panic::restore::null::open()
    }
}
