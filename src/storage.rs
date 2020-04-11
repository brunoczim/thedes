use crate::error::{ErrorExt, Result};
use chrono::Local;
use directories::ProjectDirs;
use std::{
    fmt,
    path::{Path, PathBuf},
};
use tokio::{fs, io::ErrorKind::AlreadyExists};

/// Paths to application storage.
pub fn paths() -> Result<ProjectDirs> {
    Ok(ProjectDirs::from("io.github.brunoczim", "Brunoczim", "Thedes")
        .ok_or(PathAccessError)?)
}

/// Returns the log path for the current execution, also returns the base name
/// of the file.
pub fn log_path() -> Result<(String, PathBuf)> {
    let mut path = paths()?.cache_dir().to_owned();
    let time = Local::now().format("%Y-%m-%d_%H-%M-%S%.3f");
    let name = format!("log_{}.txt", time);
    path.push(&name);
    Ok((name, path))
}

/// Ensures a directory exists.
pub async fn ensure_dir<P>(path: &P) -> Result<()>
where
    P: AsRef<Path> + ?Sized,
{
    fs::create_dir_all(path.as_ref()).await.or_else(|err| {
        if err.kind() == AlreadyExists {
            Ok(())
        } else {
            Err(err)
        }
    })?;

    Ok(())
}

/// Error triggered when application folders cannot be accessed.
#[derive(Debug)]
pub struct PathAccessError;

impl fmt::Display for PathAccessError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("Could not access application directory")
    }
}

impl ErrorExt for PathAccessError {}
