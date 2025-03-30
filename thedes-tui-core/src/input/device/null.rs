use crate::event::InternalEvent;

use super::{Error, InputDevice};

pub fn open() -> Box<dyn InputDevice> {
    Box::new(NullInputDevice)
}

#[derive(Debug, Clone, Copy)]
struct NullInputDevice;

impl InputDevice for NullInputDevice {
    fn blocking_read(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<Option<InternalEvent>, Error> {
        std::thread::sleep(timeout);
        Ok(None)
    }
}
