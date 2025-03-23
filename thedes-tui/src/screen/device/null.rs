use thedes_async_util::dyn_async_trait;

use super::{Command, Error, ScreenDevice};

pub fn open() -> Box<dyn ScreenDevice> {
    Box::new(NullScreenDevice)
}

#[derive(Debug, Clone, Copy)]
struct NullScreenDevice;

#[dyn_async_trait]
impl ScreenDevice for NullScreenDevice {
    async fn run(&mut self, command: Command) -> Result<(), Error> {
        drop(command);
        Ok(())
    }
}
