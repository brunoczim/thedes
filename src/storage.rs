use crate::error::GameResult;
use chrono::Local;
use directories::ProjectDirs;
use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};
use tokio::{fs, io::ErrorKind::AlreadyExists};

/// A game session save database related utilities.
pub mod save;

/// Error triggered when application folders cannot be accessed.
#[derive(Debug)]
pub struct PathAccessError;

impl fmt::Display for PathAccessError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("Could not access application directory")
    }
}

impl Error for PathAccessError {
    fn description(&self) -> &str {
        "Could not access application directory"
    }
}

/// Paths to application storage.
pub fn paths() -> GameResult<ProjectDirs> {
    Ok(ProjectDirs::from("io.github.brunoczim", "Brunoczim", "Thedes")
        .ok_or(PathAccessError)?)
}

/// Returns the log path for the current execution, also returns the base name
/// of the file.
pub fn log_path() -> GameResult<(String, PathBuf)> {
    let mut path = paths()?.cache_dir().to_owned();
    let time = Local::now().format("%Y-%m-%d_%H-%M-%S%.3f");
    let name = format!("log_{}.txt", time);
    path.push(&name);
    Ok((name, path))
}

/// Ensures a directory exists.
pub async fn ensure_dir<P>(path: &P) -> GameResult<()>
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
