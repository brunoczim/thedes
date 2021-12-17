use crate::{
    common::{
        block::Block,
        human::npc::{Id, InvalidId, Npc, NpcData},
        map::Coord,
        thede,
    },
    server::map::Map,
};
use gardiz::{coord::Vec2, direc::Direction};
use kopidaz::tree::Tree;

/// Storage registry for npcs.
#[derive(Debug, Clone)]
pub struct Registry {
    tree: Tree<Id, NpcData>,
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
        db: &sled::Db,
        map: &mut Map,
        head: Vec2<Coord>,
        facing: Direction,
        thede: thede::Id,
    ) -> Result<Id> {
        let data = NpcData::new(head, facing, thede);
        let res = self.tree.generate_id(
            db,
            |id| async move { Result::Ok(Id::new(id as _)) },
            |_| async move { Ok(data.clone()) },
        );

        let id = res.await?;
        map.entry_mut(data.head()).await?.block = Block::Npc(id);
        map.entry_mut(data.pointer()).await?.block = Block::Npc(id);
        Ok(id)
    }

    /// Loads the npc for a given ID.
    pub async fn load(&self, id: Id) -> Result<Npc> {
        match self.tree.get(&id).await? {
            Some(mut npc) => {
                npc.id = id;
                Ok(npc)
            },
            None => Err(InvalidId(id))?,
        }
    }

    /// Saves the given npc in storage.
    pub async fn save(&self, npc: &Npc) -> Result<()> {
        self.tree.insert(&npc.id, npc.data).await?;
        Ok(())
    }
}
