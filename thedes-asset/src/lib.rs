use std::{borrow::Cow, io, ops::Deref, path::PathBuf};

use thiserror::Error;
use tokio::sync::OnceCell;

#[derive(Debug, Error)]
#[error("Failed to load asset from {}", path.display())]
pub struct LoadError {
    pub path: PathBuf,
    pub source: io::Error,
}

#[cfg(feature = "runtime-load")]
#[doc(hidden)]
pub async fn runtime_load(
    path: impl AsRef<std::path::Path>,
) -> Result<Asset, LoadError> {
    let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    buf.push(path.as_ref());
    let bytes = tokio::fs::read(&buf)
        .await
        .map_err(|source| LoadError { path: buf.clone(), source })?;
    Ok(Asset::from_buf(bytes))
}

#[cfg(feature = "runtime-load")]
macro_rules! load {
    ($path:expr) => {
        $crate::runtime_load($path)
    };
}

#[cfg(not(feature = "runtime-load"))]
macro_rules! load {
    ($path:expr) => {
        async move {
            let bytes =
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path));
            let asset = $crate::Asset::from_static(bytes);
            Result::<_, LoadError>::Ok(asset)
        }
    };
}

#[derive(Debug, Clone)]
pub struct Asset {
    bytes: Cow<'static, [u8]>,
}

impl Asset {
    pub fn from_static(bytes: &'static [u8]) -> Self {
        Self { bytes: bytes.into() }
    }

    pub fn from_buf(bytes: Vec<u8>) -> Self {
        Self { bytes: bytes.into() }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Deref for Asset {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct SoundAssets {
    pub main_theme: Asset,
}

impl SoundAssets {
    async fn load() -> Result<Self, LoadError> {
        Ok(Self {
            main_theme: load!("../assets/audio/thedes-theme.ogg").await?,
        })
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Assets {
    pub sound: SoundAssets,
}

impl Assets {
    pub async fn get() -> Result<&'static Self, LoadError> {
        static SELF: OnceCell<Assets> = OnceCell::const_new();
        SELF.get_or_try_init(Self::load).await
    }

    async fn load() -> Result<Self, LoadError> {
        Ok(Self { sound: SoundAssets::load().await? })
    }
}
