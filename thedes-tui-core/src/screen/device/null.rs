use thedes_async_util::dyn_async_trait;

use crate::geometry::CoordPair;

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

    fn blocking_get_size(&mut self) -> Result<CoordPair, Error> {
        Ok(CoordPair { y: 1000, x: 1000 })
    }
}
