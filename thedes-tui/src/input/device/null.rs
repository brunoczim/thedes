use crate::event::InternalEvent;

use super::{Error, InputDevice};

#[derive(Debug, Clone, Copy)]
pub struct NullInputDevice;

impl InputDevice for NullInputDevice {
    fn blocking_read(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<Option<InternalEvent>, Error> {
        std::thread::sleep(timeout);
        Ok(None)
    }
}
