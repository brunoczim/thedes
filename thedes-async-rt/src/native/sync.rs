use std::ops::{Deref, DerefMut};

pub mod mpsc;
pub mod oneshot;

#[derive(Debug)]
pub struct Mutex<T> {
    inner: tokio::sync::Mutex<T>,
}

impl<T> Mutex<T> {
    pub const fn new(protected: T) -> Self {
        Self { inner: tokio::sync::Mutex::const_new(protected) }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    pub async fn lock(&self) -> MutexGuard<T> {
        MutexGuard::wrap(self.inner.lock().await)
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        self.inner.try_lock().ok().map(MutexGuard::wrap)
    }
}

#[derive(Debug)]
pub struct MutexGuard<'a, T> {
    inner: tokio::sync::MutexGuard<'a, T>,
}

impl<'a, T> MutexGuard<'a, T> {
    fn wrap(inner: tokio::sync::MutexGuard<'a, T>) -> Self {
        Self { inner }
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

#[derive(Debug)]
pub struct RwLock<T> {
    inner: tokio::sync::RwLock<T>,
}

impl<T> RwLock<T> {
    pub const fn new(protected: T) -> Self {
        Self { inner: tokio::sync::RwLock::const_new(protected) }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    pub async fn read(&self) -> RwLockReadGuard<T> {
        RwLockReadGuard::wrap(self.inner.read().await)
    }

    pub fn try_read(&self) -> Option<RwLockReadGuard<T>> {
        self.inner.try_read().ok().map(RwLockReadGuard::wrap)
    }

    pub async fn write(&self) -> RwLockWriteGuard<T> {
        RwLockWriteGuard::wrap(self.inner.write().await)
    }

    pub fn try_write(&self) -> Option<RwLockWriteGuard<T>> {
        self.inner.try_write().ok().map(RwLockWriteGuard::wrap)
    }
}

#[derive(Debug)]
pub struct RwLockReadGuard<'a, T> {
    inner: tokio::sync::RwLockReadGuard<'a, T>,
}

impl<'a, T> RwLockReadGuard<'a, T> {
    fn wrap(inner: tokio::sync::RwLockReadGuard<'a, T>) -> Self {
        Self { inner }
    }
}

impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

#[derive(Debug)]
pub struct RwLockWriteGuard<'a, T> {
    inner: tokio::sync::RwLockWriteGuard<'a, T>,
}

impl<'a, T> RwLockWriteGuard<'a, T> {
    fn wrap(inner: tokio::sync::RwLockWriteGuard<'a, T>) -> Self {
        Self { inner }
    }
}

impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}
