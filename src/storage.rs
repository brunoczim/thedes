use crate::{
    backend::{check_screen_size, Backend},
    error::GameResult,
    menu::{self, MenuItem},
    session::GameSession,
    timer,
};
use directories::ProjectDirs;
use serde::{
    de::{self, Deserialize, Deserializer, SeqAccess, Unexpected, Visitor},
    ser::{Serialize, SerializeTuple, Serializer},
};
use std::{error::Error, fmt, fs, path::PathBuf, time::Duration};

const MAGIC_NUMBER: u64 = 0x1E30_2E3A_212E_DE81;

#[derive(Debug)]
pub struct SaveName {
    path: PathBuf,
    name: String,
}

impl MenuItem for SaveName {
    fn name(&self) -> &str {
        &self.name
    }
}

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

/// Path to the saves directory.
pub fn saves_path() -> GameResult<PathBuf> {
    let dirs = paths()?;
    let mut path = PathBuf::from(dirs.data_dir());
    path.push("saves");
    Ok(path)
}

/// Lists all saves found in the saves directory.
pub fn saves() -> GameResult<Vec<SaveName>> {
    let mut list = Vec::new();

    for res in fs::read_dir(saves_path()?)? {
        let entry = res?;
        let typ = entry.file_type()?;
        if typ.is_file() || typ.is_symlink() {
            let path = entry.path();
            if let Some(name) = path.file_stem() {
                list.push(SaveName {
                    name: name.to_string_lossy().into_owned(),
                    path,
                });
            }
        }
    }

    Ok(list)
}

/// A saveable newtype over `GameSession`.
#[derive(Debug)]
pub struct Save {
    /// The session which is saved.
    pub session: GameSession,
}

impl Save {
    pub fn load<B>(backend: &mut B) -> GameResult<Self>
    where
        B: Backend,
    {
        let mut screen = backend.term_size()?;
        let saves = saves()?;
        timer::tick(Duration::from_millis(200), move || {
            check_screen_size(backend, &mut screen)?;
            unimplemented!()
        })
    }
}

impl Serialize for Save {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup_ser = serializer.serialize_tuple(2)?;

        tup_ser.serialize_element(&MAGIC_NUMBER)?;
        tup_ser.serialize_element(&self.session)?;

        Ok(tup_ser.end()?)
    }
}

#[derive(Debug)]
struct SaveVisitor;

impl<'de> Visitor<'de> for SaveVisitor {
    type Value = Save;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a magic number {} and a game session", MAGIC_NUMBER)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let magic = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &"2"))?;

        if magic != MAGIC_NUMBER {
            Err(de::Error::invalid_value(
                Unexpected::Unsigned(magic),
                &&*format!("{}", MAGIC_NUMBER),
            ))?
        }

        let session = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &"2"))?;
        Ok(Save { session })
    }
}

impl<'de> Deserialize<'de> for Save {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(deserializer.deserialize_tuple(2, SaveVisitor)?)
    }
}
