use std::{
    panic::{self, AssertUnwindSafe},
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicUsize, Ordering},
};

use futures::{FutureExt, future::BoxFuture};
use thiserror::Error;
use tokio::{
    fs,
    io::{self, AsyncWriteExt},
};

static LOCK_VERSION: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Error)]
pub enum AcquireError {
    #[error("failed to open lock")]
    OpenLock(#[source] io::Error),
    #[error("failed to write to the lock file")]
    WriteLock(#[source] io::Error),
    #[error("failed to create tables directory")]
    CreateTablesDir(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum ReleaseError {
    #[error("failed to read lock")]
    ReadLock(#[source] io::Error),
    #[error("failed to remove lock")]
    RemoveLock(#[source] io::Error),
    #[error("lock has been violated")]
    MismatchedLock,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to acquire database")]
    Acquire(#[from] AcquireError),
    #[error("failed to acquire database")]
    Release(#[from] ReleaseError),
}

#[derive(Debug)]
pub struct Database {
    base_dir: PathBuf,
    lock_content: String,
    buf: Vec<u8>,
}

impl Database {
    pub async fn with<F, T>(path: &impl AsRef<Path>, app: F) -> Result<T, Error>
    where
        F: for<'a> FnOnce(&'a mut Self) -> BoxFuture<'a, T>,
    {
        let mut this = Self::acquire(path).await?;
        let this_as_mut = &mut this;
        let app_result = AssertUnwindSafe(async { app(this_as_mut).await })
            .catch_unwind()
            .await;
        let release_result = this.release().await;
        match app_result {
            Ok(data) => {
                release_result?;
                Ok(data)
            },
            Err(panic_payload) => panic::resume_unwind(panic_payload),
        }
    }

    async fn acquire(path: impl AsRef<Path>) -> Result<Self, AcquireError> {
        let mut lock_path = path.as_ref().to_owned();
        lock_path.push(".lock");
        let mut lock_file = fs::OpenOptions::new()
            .create_new(true)
            .open(&lock_path)
            .await
            .map_err(AcquireError::OpenLock)?;
        lock_path.pop();
        let mut base_dir = lock_path;

        let pid = process::id();
        let version = LOCK_VERSION.fetch_add(1, Ordering::Relaxed);
        let lock_content = format!("{pid}:{version}");
        lock_file
            .write_all(lock_content.as_bytes())
            .await
            .map_err(AcquireError::WriteLock)?;
        lock_file.flush().await.map_err(AcquireError::WriteLock)?;
        drop(lock_file);

        let mut buf = Vec::new();

        base_dir.push("tables");
        fs::create_dir_all(&base_dir)
            .await
            .map_err(AcquireError::CreateTablesDir)?;
        base_dir.push("index.json");

        base_dir.pop();
        base_dir.pop();

        Ok(Self { base_dir, lock_content, buf })
    }

    async fn release(self) -> Result<(), ReleaseError> {
        let mut lock_path = self.base_dir;
        lock_path.push(".lock");
        let actual =
            fs::read(&lock_path).await.map_err(ReleaseError::ReadLock)?;
        if actual != self.lock_content.as_bytes() {
            Err(ReleaseError::MismatchedLock)?
        }
        fs::remove_file(&lock_path).await.map_err(ReleaseError::RemoveLock)?;
        Ok(())
    }
}
