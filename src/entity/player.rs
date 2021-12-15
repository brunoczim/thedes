use crate::{
    entity::{
        self,
        human::{self, Human},
        Physical,
    },
    error::Result,
    map::Coord,
    matter::Block,
    session::Camera,
    storage::save::SavedGame,
};
use andiskaz::{
    color::BasicColor,
    screen::Screen,
    string::TermGrapheme,
    tile::Foreground,
};
use gardiz::{coord::Vec2, direc::Direction};
use kopidaz::tree::Tree;
use rand::{rngs::StdRng, Rng};
use std::{error::Error, fmt};

const MAX_HEALTH: human::Health = 20;

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
    pub fn pointer(&self) -> Vec2<Coord> {
        self.human.pointer()
    }

    /// Coordinates of the head of this player.
    pub fn head(&self) -> Vec2<Coord> {
        self.human.head
    }

    /// Facing side of the head of this player.
    pub fn facing(&self) -> Direction {
        self.human.facing
    }

    /// Moves this player in the given direction.
    pub async fn move_around(
        &mut self,
        direc: Direction,
        game: &SavedGame,
    ) -> Result<()> {
        self.human.move_around(self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Moves this player in the given direction by quick stepping.
    pub async fn step(
        &mut self,
        direc: Direction,
        game: &SavedGame,
    ) -> Result<()> {
        self.human.step(self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Turns this player around.
    pub async fn turn_around(
        &mut self,
        direc: Direction,
        game: &SavedGame,
    ) -> Result<()> {
        self.human.turn_around(self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Renders this player on the screen.
    pub async fn render<'guard>(
        &self,
        camera: Camera,
        screen: &mut Screen<'guard>,
    ) -> Result<()> {
        self.human.render(camera, screen, &Sprite).await
    }

    pub fn health(&self) -> human::Health {
        self.human.health
    }

    pub fn max_health(&self) -> human::Health {
        self.human.max_health
    }

    async fn save(&self, game: &SavedGame) -> Result<()> {
        game.players().save(self).await
    }
}

/// Foreground of a player.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Sprite;

impl human::Sprite for Sprite {
    fn head(&self) -> Foreground {
        Foreground {
            grapheme: TermGrapheme::new_lossy("O"),
            color: BasicColor::White.into(),
        }
    }

    fn up(&self) -> Foreground {
        Foreground {
            grapheme: TermGrapheme::new_lossy("Ʌ"),
            color: BasicColor::White.into(),
        }
    }

    fn down(&self) -> Foreground {
        Foreground {
            grapheme: TermGrapheme::new_lossy("V"),
            color: BasicColor::White.into(),
        }
    }

    fn left(&self) -> Foreground {
        Foreground {
            grapheme: TermGrapheme::new_lossy("<"),
            color: BasicColor::White.into(),
        }
    }

    fn right(&self) -> Foreground {
        Foreground {
            grapheme: TermGrapheme::new_lossy(">"),
            color: BasicColor::White.into(),
        }
    }
}

/// Storage registry for players.
#[derive(Debug, Clone)]
pub struct Registry {
    tree: Tree<Id, Player>,
}

impl Registry {
    /// Creates a new registry by attempting to open a database tree.
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = Tree::open(db, "player::Registry").await?;
        Ok(Self { tree })
    }

    /// Registers a new player. Its ID is returned.
    pub async fn register(&self, game: &SavedGame) -> Result<Id> {
        let mut rng = game.seed().make_rng::<_, StdRng>(0u128);

        let low = Coord::max_value() / 5 * 2;
        let high = Coord::max_value() / 5 * 3 + Coord::max_value() % 5;
        let mut human = Human {
            head: Vec2 {
                x: rng.gen_range(low, high),
                y: rng.gen_range(low, high),
            },
            facing: Direction::Up,
            health: MAX_HEALTH,
            max_health: MAX_HEALTH,
        };

        while game.map().block(human.head).await? != Block::Empty
            || game.map().block(human.pointer()).await? != Block::Empty
        {
            human.head = Vec2 {
                x: rng.gen_range(low, high),
                y: rng.gen_range(low, high),
            };
        }

        let res = self.tree.generate_id(
            game.db(),
            |id| async move { Result::Ok(Id(id as _)) },
            |&id| {
                let player = Player { id, human: human.clone() };
                async move { Ok(player) }
            },
        );

        let id = res.await?;
        game.map()
            .set_block(human.head, Block::Entity(Physical::Player(id)))
            .await?;
        game.map()
            .set_block(human.pointer(), Block::Entity(Physical::Player(id)))
            .await?;
        Ok(id)
    }

    /// Loads the player for a given ID.
    pub async fn load(&self, id: Id) -> Result<Player> {
        match self.tree.get(&id).await? {
            Some(mut player) => {
                player.id = id;
                Ok(player)
            },
            None => Err(InvalidId(id))?,
        }
    }

    /// Saves the given player in storage.
    pub async fn save(&self, player: &Player) -> Result<()> {
        self.tree.insert(&player.id, player).await?;
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
