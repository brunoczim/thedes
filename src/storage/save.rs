use crate::{
    block,
    coord::Nat,
    error::Result,
    graphics::Grapheme,
    rand::{NoiseGen, Seed},
    storage::{ensure_dir, paths},
    ui::MenuOption,
};
use fslock::LockFile;
use std::{
    error::Error,
    fmt,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{fs, task};

/// Tests if char is valid to be put on a save name.
pub fn is_valid_name_char(ch: char) -> bool {
    ch != '/' && ch != '\\'
}

/// Must be present in a file.
pub const MAGIC_NUMBER: u64 = 0x1E30_2E3A_212E_DE81;

/// Max save name of a save.
pub const MAX_NAME: Nat = 32;

/// Extension used in a save.
pub const EXTENSION: &'static str = "thed";

/// Returns by SaveName::lock if the lock fails due to being already locked.
#[derive(Debug, Clone, Copy)]
pub struct LockFailed;

impl fmt::Display for LockFailed {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(
            "failed locking the lockfile (is there another instance of the \
             game running on that save?)",
        )
    }
}

impl Error for LockFailed {}

/// Returns by SaveName::new_game if the game already exists.
#[derive(Debug, Clone, Copy)]
pub struct AlreadyExists;

impl fmt::Display for AlreadyExists {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("save with that name already exists")
    }
}

impl Error for AlreadyExists {}

/// Returns by SaveName::new_game if the game already exists.
#[derive(Debug, Clone, Copy)]
pub struct CorruptedSave;

impl fmt::Display for CorruptedSave {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("save with that name already exists")
    }
}

impl Error for CorruptedSave {}

/// Just the name of a save.
#[derive(Debug, Clone)]
pub struct SaveName {
    path: PathBuf,
    name: Vec<Grapheme>,
}

impl SaveName {
    /// Creates a save name struct from the file stem.
    pub async fn from_stem<P>(name: &P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut path = path()?;
        ensure_dir(&path).await?;
        path.push(name.as_ref());
        path.set_extension(EXTENSION);
        let lossy = name.as_ref().to_string_lossy();
        let iter = Grapheme::iter(&lossy).filter_map(Result::ok);
        Ok(Self { name: iter.collect(), path })
    }

    /// Full path of this save.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Printable name of this save.
    pub fn printable(&self) -> &[Grapheme] {
        &self.name
    }

    /// Path of the lock file.
    pub fn lock_path(&self) -> PathBuf {
        let mut path = self.path.clone();
        path.set_extension("lock");
        path
    }

    /// Locks the lockfile for this save.
    pub async fn lock(&self) -> Result<LockFile> {
        task::block_in_place(|| {
            let mut file = LockFile::open(&self.lock_path())?;
            if !file.try_lock()? {
                Err(LockFailed)?;
            }
            Ok(file)
        })
    }

    /// Attempts to create a new game.
    pub async fn new_game(&self, seed: Seed) -> Result<SavedGame> {
        let lockfile = self.lock().await?;
        let res = task::block_in_place(|| {
            sled::Config::new().path(&self.path).create_new(true).open()
        });
        let db = match res {
            Ok(db) => db,
            Err(sled::Error::Io(err)) => {
                if err.kind() == ErrorKind::AlreadyExists {
                    Err(AlreadyExists)?
                } else {
                    Err(sled::Error::Io(err))?
                }
            },
            Err(e) => Err(e)?,
        };

        let game = SavedGame::new(lockfile, db, seed).await?;
        game.info.insert("magic", encode(MAGIC_NUMBER)?)?;

        Ok(game)
    }

    /// Attempts to create a new game.
    pub async fn load_game(&self) -> Result<SavedGame> {
        let lockfile = self.lock().await?;
        let db = task::block_in_place(|| sled::open(&self.path))?;
        Ok(SavedGame::new_existing(lockfile, db).await?)
    }

    /// Attempts to create a new game.
    pub async fn delete_game(&self) -> Result<()> {
        let _lockfile = self.lock().await?;
        fs::remove_dir_all(&self.path).await?;
        Ok(())
    }
}

impl MenuOption for SaveName {
    fn name(&self) -> Vec<Grapheme> {
        self.name.clone()
    }
}

/// Path to the saves directory.
pub fn path() -> Result<PathBuf> {
    let dirs = paths()?;
    let mut path = PathBuf::from(dirs.data_dir());
    path.push("saves");
    Ok(path)
}

/// Lists all saves found in the saves directory.
pub async fn list() -> Result<Vec<SaveName>> {
    let path = path()?;
    ensure_dir(&path).await?;

    let mut list = Vec::new();

    let mut iter = fs::read_dir(&path).await?;

    while let Some(entry) = iter.next_entry().await? {
        let typ = entry.file_type().await?;

        // Only match directories.
        if typ.is_dir() {
            let path = entry.path();
            match (path.file_stem(), path.extension()) {
                // Only match if it has save extension.
                (Some(name), Some(ext)) if ext == EXTENSION => {
                    let lossy = name.to_string_lossy();
                    let iter = Grapheme::iter(&lossy).filter_map(Result::ok);
                    list.push(SaveName { name: iter.collect(), path })
                },
                (..) => (),
            }
        }
    }

    Ok(list)
}

#[derive(Debug, Clone)]
/// A game saved on the disk.
pub struct SavedGame {
    lockfile: Arc<LockFile>,
    seed: Seed,
    info: sled::Tree,
    db: sled::Db,
    coord_fn: NoiseGen,
    block_maker: block::FromNoise,
}

impl SavedGame {
    /// Initializes coordinate noise function.
    fn coord_fn(seed: &Seed) -> NoiseGen {
        let mut noise = seed.make_noise_gen();
        noise.sensitivity = 0.005;
        noise
    }

    /// Initializes a loaded/created saved game with the given lockfile and
    /// database. If `seed` is given, it is set as the seed of the game.
    async fn new(lockfile: LockFile, db: sled::Db, seed: Seed) -> Result<Self> {
        let info = task::block_in_place(|| db.open_tree("info"))?;
        let bytes = encode(seed)?;
        task::block_in_place(|| info.insert("seed", bytes))?;

        Ok(Self {
            lockfile: Arc::new(lockfile),
            coord_fn: Self::coord_fn(&seed),
            seed,
            info,
            db,
            block_maker: block::FromNoise::new(),
        })
    }

    /// Initializes a loaded/created saved game with the given lockfile and
    /// database. If `seed` is given, it is set as the seed of the game.
    async fn new_existing(lockfile: LockFile, db: sled::Db) -> Result<Self> {
        let info = task::block_in_place(|| db.open_tree("info"))?;
        let bytes =
            task::block_in_place(|| info.get("seed"))?.ok_or(CorruptedSave)?;
        let seed = decode(&bytes)?;

        Ok(Self {
            lockfile: Arc::new(lockfile),
            coord_fn: Self::coord_fn(&seed),
            seed,
            info,
            db,
            block_maker: block::FromNoise::new(),
        })
    }
}

/// Default configs for bincode.
fn config() -> bincode::Config {
    let mut config = bincode::config();
    config.no_limit().big_endian();
    config
}

/// Encodes a value into binary.
pub fn encode<T>(val: T) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    let bytes = config().serialize(&val)?;
    Ok(bytes)
}

/// Decodes a value from binary.
pub fn decode<'de, T>(bytes: &'de [u8]) -> Result<T>
where
    T: serde::Deserialize<'de>,
{
    let val = config().deserialize(bytes)?;
    Ok(val)
}
