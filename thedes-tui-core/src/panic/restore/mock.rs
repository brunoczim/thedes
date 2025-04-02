use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::*},
    },
    thread,
};

use super::PanicRestoreGuard;

#[derive(Debug)]
struct Shared {
    open: AtomicBool,
    enabled: AtomicBool,
    called: AtomicBool,
    executed: AtomicBool,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            open: AtomicBool::new(false),
            enabled: AtomicBool::new(true),
            called: AtomicBool::new(false),
            executed: AtomicBool::new(false),
        }
    }

    pub fn mark_device_open(&self) -> bool {
        !self.open.swap(true, AcqRel)
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Acquire)
    }

    pub fn disable(&self) -> bool {
        self.enabled.swap(false, AcqRel)
    }

    pub fn called(&self) -> bool {
        self.called.load(Acquire)
    }

    pub fn mark_called(&self) -> bool {
        !self.called.swap(true, AcqRel)
    }

    pub fn executed(&self) -> bool {
        self.executed.load(Acquire)
    }

    pub fn mark_executed(&self) -> bool {
        self.mark_called();
        !self.executed.swap(true, AcqRel)
    }
}

#[derive(Debug, Clone)]
pub struct PanicRestoreMock {
    shared: Arc<Shared>,
}

impl PanicRestoreMock {
    pub fn new() -> Self {
        Self { shared: Arc::new(Shared::new()) }
    }

    pub fn open(&self) -> Box<dyn PanicRestoreGuard> {
        if !self.shared.mark_device_open() {
            panic!("Mocked input device was already open for this mock");
        }
        Box::new(MockedPanicRestoreGuard::new(self.clone()))
    }

    pub fn enabled(&self) -> bool {
        self.shared.enabled()
    }

    fn disable(&self) -> bool {
        self.shared.disable()
    }

    pub fn called(&self) -> bool {
        self.shared.called()
    }

    fn mark_called(&self) -> bool {
        self.shared.mark_called()
    }

    pub fn executed(&self) -> bool {
        self.shared.executed()
    }

    fn mark_executed(&self) -> bool {
        self.shared.mark_executed()
    }
}

#[derive(Debug)]
struct MockedPanicRestoreGuard {
    mock: PanicRestoreMock,
}

impl MockedPanicRestoreGuard {
    pub fn new(mock: PanicRestoreMock) -> Self {
        Self { mock }
    }
}

impl PanicRestoreGuard for MockedPanicRestoreGuard {
    fn cancel(self: Box<Self>) {
        self.mock.disable();
    }
}

impl Drop for MockedPanicRestoreGuard {
    fn drop(&mut self) {
        self.mock.mark_called();
        if self.mock.enabled() && thread::panicking() {
            self.mock.mark_executed();
        }
    }
}
