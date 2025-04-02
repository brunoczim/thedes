use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering::*},
};

use crate::{
    geometry::CoordPair,
    input::device::{InputDevice, mock::InputDeviceMock},
    panic::restore::{PanicRestoreGuard, mock::PanicRestoreMock},
    screen::device::{ScreenDevice, mock::ScreenDeviceMock},
};

use super::{Error, RuntimeDevice};

#[derive(Debug)]
struct Shared {
    open: AtomicBool,
    initialized: AtomicBool,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            open: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
        }
    }

    pub fn initialized(&self) -> bool {
        self.initialized.load(Acquire)
    }

    pub fn initialize(&self) -> bool {
        !self.initialized.swap(true, AcqRel)
    }

    pub fn shutdown(&self) -> bool {
        self.initialized.swap(false, AcqRel)
    }

    pub fn mark_device_open(&self) -> bool {
        !self.open.swap(true, AcqRel)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeDeviceMock {
    screen: ScreenDeviceMock,
    input: InputDeviceMock,
    panic_restore: PanicRestoreMock,
    shared: Arc<Shared>,
}

impl RuntimeDeviceMock {
    pub fn new(term_size: CoordPair) -> Self {
        Self {
            screen: ScreenDeviceMock::new(term_size),
            input: InputDeviceMock::new(),
            panic_restore: PanicRestoreMock::new(),
            shared: Arc::new(Shared::new()),
        }
    }

    pub fn initialized(&self) -> bool {
        self.shared.initialized()
    }

    fn initialize(&self) -> bool {
        self.shared.initialize()
    }

    fn shutdown(&self) -> bool {
        self.shared.shutdown()
    }

    pub fn screen(&self) -> &ScreenDeviceMock {
        &self.screen
    }

    pub fn input(&self) -> &InputDeviceMock {
        &self.input
    }

    pub fn panic_restore(&self) -> &PanicRestoreMock {
        &self.panic_restore
    }

    pub fn open(&self) -> Box<dyn RuntimeDevice> {
        if !self.shared.mark_device_open() {
            panic!("Mocked runtime device was already open for this mock");
        }
        Box::new(MockedRuntimeDevice::new(self.clone()))
    }
}

#[derive(Debug)]
struct MockedRuntimeDevice {
    mock: RuntimeDeviceMock,
}

impl MockedRuntimeDevice {
    pub fn new(mock: RuntimeDeviceMock) -> Self {
        Self { mock }
    }
}

impl RuntimeDevice for MockedRuntimeDevice {
    fn blocking_init(&mut self) -> Result<(), Error> {
        if self.mock.initialize() { Ok(()) } else { Err(Error::AlreadyInit) }
    }

    fn blocking_shutdown(&mut self) -> Result<(), Error> {
        if self.mock.shutdown() { Ok(()) } else { Err(Error::NotInit) }
    }

    fn open_input_device(&mut self) -> Box<dyn InputDevice> {
        self.mock.input().open()
    }

    fn open_screen_device(&mut self) -> Box<dyn ScreenDevice> {
        self.mock.screen().open()
    }

    fn open_panic_restore_guard(&mut self) -> Box<dyn PanicRestoreGuard> {
        self.mock.panic_restore().open()
    }
}
