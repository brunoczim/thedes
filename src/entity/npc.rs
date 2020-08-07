use crate::{
    entity::{
        self,
        human::{self, Human},
        language::Meaning,
        thede,
        Physical,
    },
    error::Result,
    graphics::{BasicColor, Foreground, GString, Grapheme},
    math::plane::{Camera, Coord2, Direc, Nat},
    matter::Block,
    storage::save::{SavedGame, Tree},
    terminal,
};
use std::{error::Error, fmt};

/// The ID of an NPC.
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

/// A handle to an NPC.
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
pub struct NPC {
    #[serde(skip)]
    #[serde(default = "dummy_id")]
    id: Id,
    human: Human,
    thede: thede::Id,
}

impl NPC {
    pub fn block(&self) -> Block {
        Block::Entity(entity::Physical::NPC(self.id))
    }

    /// Coordinates of the pointer of this npc.
    pub fn pointer(&self) -> Coord2<Nat> {
        self.human.pointer()
    }

    /// Coordinates of the head of this npc.
    pub fn head(&self) -> Coord2<Nat> {
        self.human.head
    }

    /// Facing side of the head of this npc.
    pub fn facing(&self) -> Direc {
        self.human.facing
    }

    /// Moves this npc in the given direction.
    pub async fn move_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        self.human.move_around(self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Moves this npc in the given direction by quick stepping.
    pub async fn step(&mut self, direc: Direc, game: &SavedGame) -> Result<()> {
        self.human.step(self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Turns this npc around.
    pub async fn turn_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        self.human.turn_around(self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Renders this npc on the screen.
    pub async fn render<'guard>(
        &self,
        camera: Camera,
        screen: &mut terminal::Screen<'guard>,
    ) -> Result<()> {
        self.human.render(camera, screen, &Sprite).await
    }

    /// Interacts with the player.
    pub async fn interact(
        &self,
        message: &mut GString,
        game: &SavedGame,
    ) -> Result<()> {
        let thede = game.thedes().load(self.thede).await?;
        let word_i = thede.language().word_for(Meaning::I);
        let word_exist = thede.language().word_for(Meaning::Exist);

        if let (Some(word_i), Some(word_exist)) = (word_i, word_exist) {
            *message =
                gstring![format!("{}: {} {}", self.id, word_i, word_exist)];
        }

        Ok(())
    }

    async fn save(&self, game: &SavedGame) -> Result<()> {
        game.npcs().save(self).await
    }
}

/// Default Sprite of an NPC.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Sprite;

impl human::Sprite for Sprite {
    fn head(&self) -> Foreground {
        Foreground {
            color: BasicColor::White.into(),
            grapheme: Grapheme::new_lossy("Ø"),
        }
    }

    fn up(&self) -> Foreground {
        Foreground {
            color: BasicColor::White.into(),
            grapheme: Grapheme::new_lossy("⯅"),
        }
    }

    fn down(&self) -> Foreground {
        Foreground {
            color: BasicColor::White.into(),
            grapheme: Grapheme::new_lossy("⯆"),
        }
    }

    fn left(&self) -> Foreground {
        Foreground {
            color: BasicColor::White.into(),
            grapheme: Grapheme::new_lossy("⯇"),
        }
    }

    fn right(&self) -> Foreground {
        Foreground {
            color: BasicColor::White.into(),
            grapheme: Grapheme::new_lossy("⯈"),
        }
    }
}

/// Storage registry for npcs.
#[derive(Debug, Clone)]
pub struct Registry {
    tree: Tree<Id, NPC>,
}

impl Registry {
    /// Creates a new registry by attempting to open a database tree.
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = Tree::open(db, "npc::Registry").await?;
        Ok(Self { tree })
    }

    /// Registers a new npc. Its ID is returned.
    pub async fn register(
        &self,
        game: &SavedGame,
        head: Coord2<Nat>,
        facing: Direc,
        thede: thede::Id,
    ) -> Result<Id> {
        let human = Human { head, facing };

        let res = self.tree.generate_id(
            game.db(),
            |id| Id(id as _),
            |&id| {
                let npc = NPC { id, human: human.clone(), thede };
                async move { Ok(npc) }
            },
        );

        let id = res.await?;
        game.map()
            .set_block(human.head, Block::Entity(Physical::NPC(id)))
            .await?;
        game.map()
            .set_block(human.pointer(), Block::Entity(Physical::NPC(id)))
            .await?;
        Ok(id)
    }

    /// Loads the npc for a given ID.
    pub async fn load(&self, id: Id) -> Result<NPC> {
        match self.tree.get(&id).await? {
            Some(mut npc) => {
                npc.id = id;
                Ok(npc)
            },
            None => Err(InvalidId(id))?,
        }
    }

    /// Saves the given npc in storage.
    pub async fn save(&self, npc: &NPC) -> Result<()> {
        self.tree.insert(&npc.id, npc).await?;
        Ok(())
    }
}

/// Returned by [`Registry::load`] if the NPC does not exist.
#[derive(Debug, Clone, Copy)]
pub struct InvalidId(pub Id);

impl fmt::Display for InvalidId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Invalid NPC id {}", self.0)
    }
}

impl Error for InvalidId {}
