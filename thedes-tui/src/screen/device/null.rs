use thedes_async_util::dyn_async_trait;

use super::{Command, Error, ScreenDevice};

pub fn open() -> Box<dyn ScreenDevice> {
    Box::new(NullScreenDevice)
}

#[derive(Debug, Clone, Copy)]
struct NullScreenDevice;

#[dyn_async_trait]
impl ScreenDevice for NullScreenDevice {
    fn send_raw(
        &mut self,
        commands: &mut (dyn Iterator<Item = Command> + Send + Sync),
    ) -> Result<(), Error> {
        commands.for_each(drop);
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
