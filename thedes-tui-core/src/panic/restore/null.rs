use super::PanicRestoreGuard;

pub fn open() -> Box<dyn PanicRestoreGuard> {
    Box::new(NullPanicRestoreGuard)
}

#[derive(Debug)]
struct NullPanicRestoreGuard;

impl PanicRestoreGuard for NullPanicRestoreGuard {
    fn cancel(self: Box<Self>) {}
}
