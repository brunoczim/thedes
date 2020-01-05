use crate::{
    block::{Block, BlockDist},
    entity,
    error::GameResult,
    orient::{Coord, Coord2D},
    rand::Seed,
    storage::{ensure_dir, paths},
    ui::MenuItem,
};
use fslock::LockFile;
use rand::Rng;
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
pub const MAX_NAME: Coord = 32;

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
    name: String,
}

impl SaveName {
    /// Creates a save name struct from the file stem.
    pub async fn from_stem<P>(name: &P) -> GameResult<Self>
    where
        P: AsRef<Path>,
    {
        let mut path = path()?;
        ensure_dir(&path).await?;
        path.push(name.as_ref());
        path.set_extension(EXTENSION);
        Ok(Self { name: name.as_ref().to_string_lossy().into_owned(), path })
    }

    /// Full path of this save.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Printable name of this save.
    pub fn printable(&self) -> &str {
        &self.name
    }

    pub fn lock_path(&self) -> PathBuf {
        let mut path = self.path.clone();
        path.set_extension("lock");
        path
    }

    /// Locks the lockfile for this save.
    pub async fn lock(&self) -> GameResult<LockFile> {
        task::block_in_place(|| {
            let mut file = LockFile::open(&self.lock_path())?;
            if !file.try_lock()? {
                Err(LockFailed)?;
            }
            Ok(file)
        })
    }

    /// Attempts to create a new game.
    pub async fn new_game(&self, seed: Seed) -> GameResult<SavedGame> {
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

        let (info_fut, entities_fut) = task::block_in_place(|| {
            let info = db.open_tree("info")?;
            info.insert("magic", encode(MAGIC_NUMBER)?)?;
            info.insert("seed", encode(seed)?)?;
            let entities = db.open_tree("entities")?;
            entities.insert(
                encode(entity::Id::PLAYER)?,
                encode(entity::Player::INIT)?,
            )?;
            GameResult::Ok((info.flush_async(), entities.flush_async()))
        })?;

        futures::try_join!(info_fut, entities_fut)?;

        Ok(SavedGame::new(lockfile, db).await?)
    }

    /// Attempts to create a new game.
    pub async fn load_game(&self) -> GameResult<SavedGame> {
        let lockfile = self.lock().await?;
        let db = task::block_in_place(|| sled::open(&self.path))?;
        Ok(SavedGame::new(lockfile, db).await?)
    }

    /// Attempts to create a new game.
    pub async fn delete_game(&self) -> GameResult<()> {
        let _lockfile = self.lock().await?;
        fs::remove_dir_all(&self.path).await?;
        Ok(())
    }
}

impl MenuItem for SaveName {
    fn name(&self) -> &str {
        &self.name
    }
}

/// Path to the saves directory.
pub fn path() -> GameResult<PathBuf> {
    let dirs = paths()?;
    let mut path = PathBuf::from(dirs.data_dir());
    path.push("saves");
    Ok(path)
}

/// Lists all saves found in the saves directory.
pub async fn list() -> GameResult<Vec<SaveName>> {
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
                    list.push(SaveName {
                        name: name.to_string_lossy().into_owned(),
                        path,
                    })
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
    db: sled::Db,
}

impl SavedGame {
    async fn new(lockfile: LockFile, db: sled::Db) -> GameResult<Self> {
        let res = task::block_in_place(|| {
            GameResult::Ok(
                db.open_tree("info")?.get("seed")?.ok_or(CorruptedSave)?,
            )
        });
        let seed = decode(&res?)?;
        Ok(Self { lockfile: Arc::new(lockfile), seed, db })
    }

    /// Returns the contents of a block at the given coordinates.
    pub async fn block_at(&self, coord: Coord2D) -> GameResult<Block> {
        let res = task::block_in_place(|| {
            let tree = self.db.open_tree("blocks")?;
            let vec = tree.get(encode(coord)?)?;
            GameResult::Ok(vec)
        });
        match res? {
            Some(bytes) => Ok(decode(&bytes)?),
            None => Ok(self.seed.make_rng(coord).sample(BlockDist)),
        }
    }

    /// Sets the contents of a block at the given coordinates.
    pub async fn set_block_at(
        &self,
        coord: Coord2D,
        block: Block,
    ) -> GameResult<()> {
        let coord_vec = encode(coord)?;
        let block_vec = encode(block)?;
        task::block_in_place(|| {
            let tree = self.db.open_tree("blocks")?;
            tree.insert(coord_vec, block_vec)?;
            Ok(())
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
pub fn encode<T>(val: T) -> GameResult<Vec<u8>>
where
    T: serde::Serialize,
{
    let bytes = config().serialize(&val)?;
    Ok(bytes)
}

/// Decodes a value from binary.
pub fn decode<'de, T>(bytes: &'de [u8]) -> GameResult<T>
where
    T: serde::Deserialize<'de>,
{
    let val = config().deserialize(bytes)?;
    Ok(val)
}
