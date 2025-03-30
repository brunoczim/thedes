use std::time::Duration;

use crate::event::{InternalEvent, native_ext::FromCrossterm};

use super::{Error, InputDevice};

pub fn open() -> Box<dyn InputDevice> {
    Box::new(NativeInputDevice)
}

#[derive(Debug)]
struct NativeInputDevice;

impl InputDevice for NativeInputDevice {
    fn blocking_read(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<InternalEvent>, Error> {
        if crossterm::event::poll(timeout)? {
            let crossterm_event = crossterm::event::read()?;
            Ok(InternalEvent::from_crossterm(crossterm_event))
        } else {
            Ok(None)
        }
    }
}
