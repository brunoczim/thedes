use std::time::Duration;

use crate::event::{Event, InternalEvent, native_ext::FromCrossterm};

use super::{Error, InputDevice};

#[derive(Debug)]
pub struct NativeInputDevice {
    _private: (),
}

impl NativeInputDevice {
    pub fn open() -> Self {
        Self { _private: () }
    }
}

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
