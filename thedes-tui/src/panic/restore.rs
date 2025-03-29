use std::fmt;

pub mod native;
pub mod null;

pub trait PanicRestoreGuard: fmt::Debug + Send + Sync {
    fn cancel(self: Box<Self>);
}

impl<T> PanicRestoreGuard for Box<T>
where
    T: PanicRestoreGuard + ?Sized,
{
    fn cancel(self: Box<Self>) {
        T::cancel(*self);
    }
}
