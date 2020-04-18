use crate::{
    coord::{Camera, Coord2, Direc, Nat},
    entity::{
        self,
        human::{self, Human},
        Physical,
    },
    error::Result,
    graphics::{Color, Foreground, Grapheme},
    matter::{block, Block},
    rand::Seed,
    storage::save::{self, SavedGame},
    terminal,
};
use rand::{rngs::StdRng, Rng};
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
        Block::Entity(entity::Physical::Player(self.id))
    }

    /// Coordinates of the pointer of this player.
    pub fn pointer(&self) -> Coord2<Nat> {
        self.human.pointer()
    }

    /// Coordinates of the head of this player.
    pub fn head(&self) -> Coord2<Nat> {
        self.human.head
    }

    /// Facing side of the head of this player.
    pub fn facing(&self) -> Direc {
        self.human.facing
    }

    /// Moves this player in the given direction.
    pub async fn move_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        self.human.move_around(&self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Moves this player in the given direction by quick stepping.
    pub async fn step(&mut self, direc: Direc, game: &SavedGame) -> Result<()> {
        self.human.step(&self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Turns this player around.
    pub async fn turn_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        self.human.turn_around(&self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Renders this player on the screen.
    pub async fn render<'guard>(
        &self,
        camera: Camera,
        screen: &mut terminal::Screen<'guard>,
    ) -> Result<()> {
        self.human.render(camera, screen, &Sprite).await
    }

    async fn save(&self, game: &SavedGame) -> Result<()> {
        game.players().save(self).await
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

/// Storage registry for players.
#[derive(Debug, Clone)]
pub struct Registry {
    tree: sled::Tree,
}

impl Registry {
    /// Creates a new registry by attempting to open a database tree.
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = task::block_in_place(|| db.open_tree("player::Registry"))?;
        Ok(Self { tree })
    }

    /// Registers a new player. Its ID is returned.
    pub async fn register(
        &self,
        db: &sled::Db,
        map: &block::Map,
        seed: Seed,
    ) -> Result<Id> {
        let mut rng = seed.make_rng::<_, StdRng>(0u128);

        let low = Nat::max_value() / 5 * 2;
        let high = Nat::max_value() / 5 * 3 + Nat::max_value() % 5;
        let mut human = Human {
            head: Coord2 {
                x: rng.gen_range(low, high),
                y: rng.gen_range(low, high),
            },
            facing: Direc::Up,
        };

        while map.get(human.head).await? != Block::Empty
            || map.get(human.pointer()).await? != Block::Empty
        {
            human.head = Coord2 {
                x: rng.gen_range(low, high),
                y: rng.gen_range(low, high),
            };
        }

        let res = save::generate_id(
            db,
            &self.tree,
            |id| Id(id as _),
            |&id| {
                let player = Player { id, human: human.clone() };
                async move { Ok(player) }
            },
        );

        let id = res.await?;
        map.set(human.head, &Block::Entity(Physical::Player(id))).await?;
        map.set(human.pointer(), &Block::Entity(Physical::Player(id))).await?;
        Ok(id)
    }

    /// Loads the player for a given ID.
    pub async fn load(&self, id: Id) -> Result<Player> {
        let id_vec = save::encode(id)?;
        let res = task::block_in_place(|| self.tree.get(id_vec));

        match res? {
            Some(data) => {
                let mut player = save::decode::<Player>(&data)?;
                player.id = id;
                Ok(player)
            },
            None => Err(InvalidId(id))?,
        }
    }

    /// Saves the given player in storage.
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
