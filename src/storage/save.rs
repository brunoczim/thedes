use crate::{
    entity::{biome, npc, player, thede},
    error::Result,
    graphics::GString,
    map::{Map, RECOMMENDED_CACHE_LIMIT},
    math::{plane::Nat, rand::Seed},
    matter::{block, ground},
    storage::{ensure_dir, paths},
    ui::MenuOption,
};
use fslock::LockFile;
use std::{
    error::Error,
    fmt,
    future::Future,
    io::ErrorKind,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{fs, sync::Mutex, task};

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
        fmt.write_str("this save is likely corrupted")
    }
}

impl Error for CorruptedSave {}

/// Just the name of a save.
#[derive(Debug, Clone)]
pub struct SaveName {
    path: PathBuf,
    printable: GString,
}

impl SaveName {
    /// Creates a save name struct from the file stem.
    pub async fn from_stem(stem: GString) -> Result<Self> {
        let mut path = path()?;
        ensure_dir(&path).await?;
        path.push(&stem);
        path.set_extension(EXTENSION);
        Ok(Self { printable: stem, path })
    }

    /// Full path of this save.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Printable name of this save.
    pub fn printable(&self) -> &GString {
        &self.printable
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
    fn name(&self) -> GString {
        self.printable.clone()
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
                    let printable = GString::new_lossy(name.to_string_lossy());
                    list.push(SaveName { printable, path })
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
    map: Arc<Mutex<Map>>,
    players: player::Registry,
    npcs: npc::Registry,
    thedes: thede::Registry,
}

impl SavedGame {
    /// Initializes a loaded/created saved game with the given lockfile and
    /// database. If `seed` is given, it is set as the seed of the game.
    async fn new(lockfile: LockFile, db: sled::Db, seed: Seed) -> Result<Self> {
        let info = task::block_in_place(|| db.open_tree("info"))?;
        let bytes = encode(seed)?;
        task::block_in_place(|| info.insert("seed", bytes))?;
        let thedes = thede::Registry::new(&db).await?;
        let players = player::Registry::new(&db).await?;
        let mut map = Map::new(RECOMMENDED_CACHE_LIMIT, db, seed);
        let player = players.register(&db, &mut map, seed).await?;
        let npcs = npc::Registry::new(&db).await?;
        let bytes = encode(player)?;
        task::block_in_place(|| info.insert("default_player", bytes))?;

        Ok(Self {
            lockfile: Arc::new(lockfile),
            thedes,
            map: Arc::new(Mutex::new(map)),
            players,
            npcs,
            seed,
            info,
            db,
        })
    }

    /// Initializes a loaded/created saved game with the given lockfile and
    /// database. If `seed` is given, it is set as the seed of the game.
    async fn new_existing(lockfile: LockFile, db: sled::Db) -> Result<Self> {
        let info = task::block_in_place(|| db.open_tree("info"))?;
        let bytes =
            task::block_in_place(|| info.get("seed"))?.ok_or(CorruptedSave)?;
        let seed = decode(&bytes)?;
        let thedes = thede::Registry::new(&db).await?;
        let map = Map::new(RECOMMENDED_CACHE_LIMIT, db, seed);
        let players = player::Registry::new(&db).await?;
        let npcs = npc::Registry::new(&db).await?;

        Ok(Self {
            lockfile: Arc::new(lockfile),
            thedes,
            map: Arc::new(Mutex::new(map)),
            players,
            npcs,
            seed,
            info,
            db,
        })
    }

    /// Returns the seed of this save.
    pub fn seed(&self) -> Seed {
        self.seed
    }

    /// Returns the underlying database.
    pub fn db(&self) -> &sled::Db {
        &self.db
    }

    /// Gives access to the map of blocks.
    pub fn map(&self) -> &Mutex<Map> {
        &self.map
    }

    /// Gives access to the registry of thedes.
    pub fn thedes(&self) -> &thede::Registry {
        &self.thedes
    }

    /// Gives access to the registry of players.
    pub fn players(&self) -> &player::Registry {
        &self.players
    }

    /// Gives access to the registry of NPCs.
    pub fn npcs(&self) -> &npc::Registry {
        &self.npcs
    }

    /// Returns the ID of the default player.
    pub async fn default_player(&self) -> Result<player::Id> {
        let bytes = self.info.get("default_player")?.ok_or(CorruptedSave)?;
        Ok(decode(&bytes)?)
    }
}

/// A persistent key structure.
pub struct Tree<K, V>
where
    for<'de> K: serde::Serialize + serde::Deserialize<'de>,
    for<'de> V: serde::Serialize + serde::Deserialize<'de>,
{
    storage: sled::Tree,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> Tree<K, V>
where
    for<'de> K: serde::Serialize + serde::Deserialize<'de>,
    for<'de> V: serde::Serialize + serde::Deserialize<'de>,
{
    /// Opens this tree from a database.
    pub async fn open<T>(db: &sled::Db, name: T) -> Result<Self>
    where
        T: AsRef<[u8]>,
    {
        let storage = task::block_in_place(|| db.open_tree(name))?;
        Ok(Self { storage, _marker: PhantomData })
    }

    /// Gets a value associated with a given key.
    pub async fn get(&self, key: &K) -> Result<Option<V>> {
        let encoded_key = encode(key)?;
        let maybe = task::block_in_place(|| self.storage.get(&encoded_key))?;
        match maybe {
            Some(encoded_val) => {
                let val = decode(&encoded_val)?;
                Ok(Some(val))
            },
            None => Ok(None),
        }
    }

    /// Inserts a value associated with a given key.
    pub async fn insert(&self, key: &K, val: &V) -> Result<()> {
        let encoded_key = encode(key)?;
        let encoded_val = encode(val)?;
        task::block_in_place(|| {
            self.storage.insert(&encoded_key, encoded_val)
        })?;
        Ok(())
    }

    /// Returns whether the given key is present in this tree.
    pub async fn contains_key(&self, key: &K) -> Result<bool> {
        let encoded_key = encode(key)?;
        let result =
            task::block_in_place(|| self.storage.contains_key(&encoded_key))?;
        Ok(result)
    }

    /// Tries to generate an ID until it is successful. The ID is stored
    /// alongside with a value in a given tree.
    pub async fn generate_id<F, G, A>(
        &self,
        db: &sled::Db,
        mut make_id: F,
        make_data: G,
    ) -> Result<K>
    where
        F: FnMut(u64) -> K,
        G: FnOnce(&K) -> A,
        A: Future<Output = Result<V>>,
    {
        let mut attempt = 0usize;
        loop {
            let generated = task::block_in_place(|| db.generate_id())?;
            let id = make_id(generated);

            let contains = self.contains_key(&id).await?;

            if !contains {
                let data = make_data(&id).await?;
                self.insert(&id, &data).await?;
                break Ok(id);
            }

            if attempt == 20 {
                task::yield_now().await;
                attempt = 0;
            } else {
                attempt += 1;
            }
        }
    }
}

impl<K, V> Clone for Tree<K, V>
where
    for<'de> K: serde::Serialize + serde::Deserialize<'de>,
    for<'de> V: serde::Serialize + serde::Deserialize<'de>,
{
    fn clone(&self) -> Self {
        Self { storage: self.storage.clone(), _marker: PhantomData }
    }
}

impl<K, V> fmt::Debug for Tree<K, V>
where
    for<'de> K: serde::Serialize + serde::Deserialize<'de>,
    for<'de> V: serde::Serialize + serde::Deserialize<'de>,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Tree").field("storage", &self.storage).finish()
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
