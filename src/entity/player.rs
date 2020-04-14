use crate::{
    coord::{Camera, Coord2, Direc, Nat},
    entity::human::{self, Human},
    error::Result,
    graphics::{Color, Foreground, Grapheme},
    matter::Block,
    storage::save::{self, SavedGame},
    terminal,
};
use std::{error::Error, fmt};
use tokio::task;

/// The ID of a player.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Id(u32);

fn dummy_id() -> Id {
    Id(0)
}

impl fmt::Display for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:x}", self.0)
    }
}

/// A handle to the player.
#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Player {
    #[serde(skip)]
    #[serde(default = "dummy_id")]
    id: Id,
    human: Human,
}

impl Player {
    pub fn block(&self) -> Block {
        Block::Player(self.id)
    }

    /// Coordinates of the pointer of this human.
    pub fn pointer(&self) -> Coord2<Nat> {
        self.human.pointer()
    }

    /// Moves this human in the given direction.
    pub async fn move_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        self.human.move_around(&self.block(), direc, game).await
    }

    /// Moves this human in the given direction by quick stepping.
    pub async fn step(&mut self, direc: Direc, game: &SavedGame) -> Result<()> {
        self.human.step(&self.block(), direc, game).await
    }

    /// Turns this human around.
    pub async fn turn_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        self.human.turn_around(&self.block(), direc, game).await
    }

    /// Renders this human on the screen, with the given sprite.
    pub async fn render<'guard>(
        &self,
        camera: Camera,
        screen: &mut terminal::Screen<'guard>,
    ) -> Result<()> {
        self.human.render(camera, screen, &Sprite).await
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Sprite;

impl human::Sprite for Sprite {
    fn head(&self) -> Foreground {
        Foreground { color: Color::White, grapheme: Grapheme::new_lossy("O") }
    }

    fn up(&self) -> Foreground {
        Foreground { color: Color::White, grapheme: Grapheme::new_lossy("Ʌ") }
    }

    fn down(&self) -> Foreground {
        Foreground { color: Color::White, grapheme: Grapheme::new_lossy("V") }
    }

    fn left(&self) -> Foreground {
        Foreground { color: Color::White, grapheme: Grapheme::new_lossy("<") }
    }

    fn right(&self) -> Foreground {
        Foreground { color: Color::White, grapheme: Grapheme::new_lossy(">") }
    }
}

#[derive(Debug, Clone)]
pub struct Registry {
    tree: sled::Tree,
}

impl Registry {
    pub fn new(db: &sled::Db) -> Result<Self> {
        let tree = task::block_in_place(|| db.open_tree("player::Registry"))?;
        Ok(Self { tree })
    }

    pub async fn register(&self, db: &sled::Db) -> Result<Id> {
        let res = save::generate_id(
            db,
            &self.tree,
            |id| Id(id as _),
            |id| Player {
                id: *id,
                human: Human { head: Coord2 { x: 0, y: 0 }, facing: Direc::Up },
            },
        );

        res.await
    }

    pub async fn load(&self, id: Id) -> Result<Player> {
        let id_vec = save::encode(id)?;
        let mut res = task::block_in_place(|| self.tree.get(id_vec));

        match res? {
            Some(data) => {
                let mut player = save::decode::<Player>(&data)?;
                player.id = id;
                Ok(player)
            },
            None => Err(InvalidId(id))?,
        }
    }

    pub async fn save(&self, player: &Player) -> Result<()> {
        let id_vec = save::encode(player.id)?;
        let data_vec = save::encode(player)?;
        task::block_in_place(|| self.tree.insert(id_vec, data_vec))?;
        Ok(())
    }
}

/// Returned by [`Registry::load`] if the player does not exist.
#[derive(Debug, Clone, Copy)]
pub struct InvalidId(pub Id);

impl fmt::Display for InvalidId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Invalid player id {}", self.0)
    }
}

impl Error for InvalidId {}
