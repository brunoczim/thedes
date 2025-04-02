use std::fmt;

use thiserror::Error;
use tokio::io;

use crate::{
    input::device::InputDevice,
    panic::restore::PanicRestoreGuard,
    screen::device::ScreenDevice,
};

pub mod native;
pub mod null;

#[cfg(feature = "testing")]
pub mod mock;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Already initialized")]
    AlreadyInit,
    #[error("Not initialized yet or already shut down")]
    NotInit,
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub trait RuntimeDevice: fmt::Debug + Send + Sync {
    fn blocking_init(&mut self) -> Result<(), Error>;

    fn blocking_shutdown(&mut self) -> Result<(), Error>;

    fn open_screen_device(&mut self) -> Box<dyn ScreenDevice>;

    fn open_input_device(&mut self) -> Box<dyn InputDevice>;

    fn open_panic_restore_guard(&mut self) -> Box<dyn PanicRestoreGuard>;
}

impl<'a, T> RuntimeDevice for &'a mut T
where
    T: RuntimeDevice + ?Sized,
{
    fn blocking_init(&mut self) -> Result<(), Error> {
        (**self).blocking_init()
    }

    fn blocking_shutdown(&mut self) -> Result<(), Error> {
        (**self).blocking_shutdown()
    }

    fn open_screen_device(&mut self) -> Box<dyn ScreenDevice> {
        (**self).open_screen_device()
    }

    fn open_input_device(&mut self) -> Box<dyn InputDevice> {
        (**self).open_input_device()
    }

    fn open_panic_restore_guard(&mut self) -> Box<dyn PanicRestoreGuard> {
        (**self).open_panic_restore_guard()
    }
}

impl<T> RuntimeDevice for Box<T>
where
    T: RuntimeDevice + ?Sized,
{
    fn blocking_init(&mut self) -> Result<(), Error> {
        (**self).blocking_init()
    }

    fn blocking_shutdown(&mut self) -> Result<(), Error> {
        (**self).blocking_shutdown()
    }

    fn open_screen_device(&mut self) -> Box<dyn ScreenDevice> {
        (**self).open_screen_device()
    }

    fn open_input_device(&mut self) -> Box<dyn InputDevice> {
        (**self).open_input_device()
    }

    fn open_panic_restore_guard(&mut self) -> Box<dyn PanicRestoreGuard> {
        (**self).open_panic_restore_guard()
    }
}
