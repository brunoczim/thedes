use std::ops::{Deref, DerefMut};

use crate::backend;

pub mod mpsc;

#[derive(Debug)]
pub struct Mutex<T> {
    inner: backend::sync::Mutex<T>,
}

impl<T> Mutex<T> {
    pub const fn new(protected: T) -> Self {
        Self { inner: backend::sync::Mutex::new(protected) }
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
        self.inner.try_lock().map(MutexGuard::wrap)
    }
}

#[derive(Debug)]
pub struct MutexGuard<'a, T> {
    inner: backend::sync::MutexGuard<'a, T>,
}

impl<'a, T> MutexGuard<'a, T> {
    fn wrap(inner: backend::sync::MutexGuard<'a, T>) -> Self {
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
    inner: backend::sync::RwLock<T>,
}

impl<T> RwLock<T> {
    pub const fn new(protected: T) -> Self {
        Self { inner: backend::sync::RwLock::new(protected) }
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
        self.inner.try_read().map(RwLockReadGuard::wrap)
    }

    pub async fn write(&self) -> RwLockWriteGuard<T> {
        RwLockWriteGuard::wrap(self.inner.write().await)
    }

    pub fn try_write(&self) -> Option<RwLockWriteGuard<T>> {
        self.inner.try_write().map(RwLockWriteGuard::wrap)
    }
}

#[derive(Debug)]
pub struct RwLockReadGuard<'a, T> {
    inner: backend::sync::RwLockReadGuard<'a, T>,
}

impl<'a, T> RwLockReadGuard<'a, T> {
    fn wrap(inner: backend::sync::RwLockReadGuard<'a, T>) -> Self {
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
    inner: backend::sync::RwLockWriteGuard<'a, T>,
}

impl<'a, T> RwLockWriteGuard<'a, T> {
    fn wrap(inner: backend::sync::RwLockWriteGuard<'a, T>) -> Self {
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
