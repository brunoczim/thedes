use crate::{
    entity::{
        self,
        human::{self, Human},
        language::Meaning,
        thede,
        Physical,
    },
    error::Result,
    graphics::{Color, ContrastiveFg, GString, Grapheme, Tile},
    math::plane::{Camera, Coord2, Direc, Nat},
    matter::{block, Block},
    storage::save::{self, SavedGame},
    terminal,
};
use std::{error::Error, fmt};
use tokio::task;

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
        self.human.move_around(&self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Moves this npc in the given direction by quick stepping.
    pub async fn step(&mut self, direc: Direc, game: &SavedGame) -> Result<()> {
        self.human.step(&self.block(), direc, game).await?;
        self.save(game).await
    }

    /// Turns this npc around.
    pub async fn turn_around(
        &mut self,
        direc: Direc,
        game: &SavedGame,
    ) -> Result<()> {
        self.human.turn_around(&self.block(), direc, game).await?;
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
    fn head(&self) -> Tile<ContrastiveFg> {
        Tile {
            colors: ContrastiveFg { color: Color::White },
            grapheme: Grapheme::new_lossy("Ø"),
        }
    }

    fn up(&self) -> Tile<ContrastiveFg> {
        Tile {
            colors: ContrastiveFg { color: Color::White },
            grapheme: Grapheme::new_lossy("⯅"),
        }
    }

    fn down(&self) -> Tile<ContrastiveFg> {
        Tile {
            colors: ContrastiveFg { color: Color::White },
            grapheme: Grapheme::new_lossy("⯆"),
        }
    }

    fn left(&self) -> Tile<ContrastiveFg> {
        Tile {
            colors: ContrastiveFg { color: Color::White },
            grapheme: Grapheme::new_lossy("⯇"),
        }
    }

    fn right(&self) -> Tile<ContrastiveFg> {
        Tile {
            colors: ContrastiveFg { color: Color::White },
            grapheme: Grapheme::new_lossy("⯈"),
        }
    }
}

/// Storage registry for npcs.
#[derive(Debug, Clone)]
pub struct Registry {
    tree: sled::Tree,
}

impl Registry {
    /// Creates a new registry by attempting to open a database tree.
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = task::block_in_place(|| db.open_tree("npc::Registry"))?;
        Ok(Self { tree })
    }

    /// Registers a new npc. Its ID is returned.
    pub async fn register(
        &self,
        db: &sled::Db,
        map: &block::Map,
        head: Coord2<Nat>,
        facing: Direc,
        thede: thede::Id,
    ) -> Result<Id> {
        let human = Human { head, facing };

        let res = save::generate_id(
            db,
            &self.tree,
            |id| Id(id as _),
            |&id| {
                let npc = NPC { id, human: human.clone(), thede };
                async move { Ok(npc) }
            },
        );

        let id = res.await?;
        map.set(human.head, &Block::Entity(Physical::NPC(id))).await?;
        map.set(human.pointer(), &Block::Entity(Physical::NPC(id))).await?;
        Ok(id)
    }

    /// Loads the npc for a given ID.
    pub async fn load(&self, id: Id) -> Result<NPC> {
        let id_vec = save::encode(id)?;
        let res = task::block_in_place(|| self.tree.get(id_vec));

        match res? {
            Some(data) => {
                let mut npc = save::decode::<NPC>(&data)?;
                npc.id = id;
                Ok(npc)
            },
            None => Err(InvalidId(id))?,
        }
    }

    /// Saves the given npc in storage.
    pub async fn save(&self, npc: &NPC) -> Result<()> {
        let id_vec = save::encode(npc.id)?;
        let data_vec = save::encode(npc)?;
        task::block_in_place(|| self.tree.insert(id_vec, data_vec))?;
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
