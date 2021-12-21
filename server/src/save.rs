use crate::{
    language,
    map::{Map, Navigator, RECOMMENDED_CACHE_LIMIT},
    npc,
    player,
    random::random_seed,
    thede,
};
use fslock::LockFile;
use std::{error::Error as StdError, ffi::OsStr, fmt, path::PathBuf};
use thedes_common::{seed::Seed, version::Version, Result, ResultExt};
use tokio::task;

#[derive(Debug, Clone)]
struct LockFailed {
    path: PathBuf,
}

impl fmt::Display for LockFailed {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmtr,
            "save {} is already locked, perhaps someone else is still using it",
            self.path.display()
        )
    }
}

impl StdError for LockFailed {}

#[derive(Debug, Clone)]
struct UndefinedSeed;

impl fmt::Display for UndefinedSeed {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "seed is undefined",)
    }
}

impl StdError for UndefinedSeed {}

#[derive(Debug, Clone)]
struct UndefinedVersion;

impl fmt::Display for UndefinedVersion {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "version is undefined",)
    }
}

impl StdError for UndefinedVersion {}

#[derive(Debug, Clone)]
struct LockPath {
    buffer: PathBuf,
}

impl LockPath {
    fn new<P, S>(saves_dir: P, save_name: S) -> Self
    where
        P: Into<PathBuf>,
        S: AsRef<OsStr>,
    {
        let mut full_path = saves_dir.into();
        full_path.push(save_name.as_ref());

        let mut path_as_string = full_path.into_os_string();
        path_as_string.push(".thlk");

        Self { buffer: path_as_string.into() }
    }

    async fn lock_save(self) -> Result<SavePath> {
        task::block_in_place(|| {
            let mut lockfile = LockFile::open(&self.buffer).erase_err()?;
            if !lockfile.try_lock_with_pid().erase_err()? {
                return Err(LockFailed { path: self.buffer }).erase_err();
            }
            let mut buffer = self.buffer;
            buffer.set_extension(".thdb");
            Ok(SavePath { buffer, lockfile })
        })
    }
}

#[derive(Debug)]
struct SavePath {
    buffer: PathBuf,
    lockfile: LockFile,
}

impl SavePath {
    async fn open(self) -> Result<(sled::Db, LockFile)> {
        let db = kopidaz::open(&self.buffer).await.erase_err()?;
        Ok((db, self.lockfile))
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    tree: sled::Tree,
}

impl Info {
    async fn new(db: &sled::Db) -> Result<Self> {
        Ok(Self {
            tree: task::block_in_place(|| db.open_tree("info")).erase_err()?,
        })
    }

    pub async fn seed(&self) -> Result<Seed> {
        let bytes = task::block_in_place(|| self.tree.get("seed"))
            .erase_err()?
            .ok_or(UndefinedSeed)
            .erase_err()?;
        kopidaz::decode(&bytes).erase_err()
    }

    pub async fn set_seed(&mut self, seed: Seed) -> Result<Option<Seed>> {
        let encoded = kopidaz::encode(seed).erase_err()?;
        match task::block_in_place(|| self.tree.insert("seed", encoded))
            .erase_err()?
        {
            Some(bytes) => {
                let previous = kopidaz::decode(&bytes).erase_err()?;
                Ok(Some(previous))
            },
            None => Ok(None),
        }
    }

    pub async fn version(&self) -> Result<Version> {
        let bytes = task::block_in_place(|| self.tree.get("version"))
            .erase_err()?
            .ok_or(UndefinedVersion)
            .erase_err()?;
        kopidaz::decode(&bytes).erase_err()
    }

    pub async fn set_version(
        &mut self,
        version: Version,
    ) -> Result<Option<Version>> {
        let encoded = kopidaz::encode(version).erase_err()?;
        match task::block_in_place(|| self.tree.insert("version", encoded))
            .erase_err()?
        {
            Some(bytes) => {
                let previous = kopidaz::decode(&bytes).erase_err()?;
                Ok(Some(previous))
            },
            None => Ok(None),
        }
    }
}

#[derive(Debug)]
pub struct SavedGame {
    _lockfile: LockFile,
    pub db: sled::Db,
    pub info: Info,
    pub map: Navigator,
    pub players: player::Registry,
    pub npcs: npc::Registry,
    pub languages: language::Registry,
    pub thedes: thede::Registry,
}

impl SavedGame {
    pub async fn create<P, S>(saves_dir: P, save_name: S) -> Result<Self>
    where
        P: Into<PathBuf>,
        S: AsRef<OsStr>,
    {
        let (db, lockfile) = LockPath::new(saves_dir, save_name)
            .lock_save()
            .await?
            .open()
            .await?;

        let mut info = Info::new(&db).await?;
        info.set_version(Version::CURRENT).await?;
        info.set_seed(random_seed()).await?;

        let map = Navigator::new(Map::new(&db, RECOMMENDED_CACHE_LIMIT).await?);
        let players = player::Registry::new(&db).await?;
        let npcs = npc::Registry::new(&db).await?;
        let languages = language::Registry::new(&db).await?;
        let thedes = thede::Registry::new(&db).await?;

        Ok(Self {
            _lockfile: lockfile,
            db,
            info,
            map,
            players,
            npcs,
            languages,
            thedes,
        })
    }

    pub async fn load<P, S>(saves_dir: P, save_name: S) -> Result<Self>
    where
        P: Into<PathBuf>,
        S: AsRef<OsStr>,
    {
        let (db, lockfile) = LockPath::new(saves_dir, save_name)
            .lock_save()
            .await?
            .open()
            .await?;

        let info = Info::new(&db).await?;
        let map = Navigator::new(Map::new(&db, RECOMMENDED_CACHE_LIMIT).await?);
        let players = player::Registry::new(&db).await?;
        let npcs = npc::Registry::new(&db).await?;
        let languages = language::Registry::new(&db).await?;
        let thedes = thede::Registry::new(&db).await?;

        Ok(Self {
            _lockfile: lockfile,
            db,
            info,
            map,
            players,
            npcs,
            languages,
            thedes,
        })
    }
}
