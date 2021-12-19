use crate::{human, map::Map, thede};
use gardiz::{coord::Vec2, direc::Direction};
use kopidaz::tree::Tree;
use thedes_common::{
    block::Block,
    error::{BadNpcId, Error},
    map::Coord,
    Result,
    ResultExt,
};

pub use thedes_common::npc::{Data, Id, Npc, MAX_HEALTH};

pub async fn move_around(
    npc: &mut Npc,
    direc: Direction,
    map: &mut Map,
    registry: &Registry,
) -> Result<()> {
    let block = npc.block();
    human::move_around(&mut npc.data.body, block, direc, map).await?;
    registry.save(*npc).await?;
    Ok(())
}

pub async fn step(
    npc: &mut Npc,
    direc: Direction,
    map: &mut Map,
    registry: &Registry,
) -> Result<()> {
    let block = npc.block();
    human::step(&mut npc.data.body, block, direc, map).await?;
    registry.save(*npc).await?;
    Ok(())
}

pub async fn turn_around(
    npc: &mut Npc,
    direc: Direction,
    map: &mut Map,
    registry: &Registry,
) -> Result<()> {
    let block = npc.block();
    human::turn_around(&mut npc.data.body, block, direc, map).await?;
    registry.save(*npc).await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Registry {
    tree: Tree<Id, Data>,
}

impl Registry {
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = Tree::open(db, "npc::Registry").await.erase_err()?;
        Ok(Self { tree })
    }

    pub async fn register(
        &self,
        head: Vec2<Coord>,
        facing: Direction,
        thede: thede::Id,
        db: &sled::Db,
        map: &mut Map,
    ) -> Result<Npc> {
        let (id, data) = self
            .tree
            .id_builder()
            .error_conversor(Error::erase)
            .id_maker(|bits| Id(bits as _))
            .data_maker(|_| Data {
                body: human::Body { head, facing },
                health: MAX_HEALTH,
                max_health: MAX_HEALTH,
                thede,
            })
            .generate(db)
            .await?;

        human::write_on_map(data.body, Block::Npc(id), map).await?;
        Ok(Npc { id, data })
    }

    pub async fn load(&self, id: Id) -> Result<Npc> {
        match self.tree.get(&id).await.erase_err()? {
            Some(data) => Ok(Npc { id, data }),
            None => Err(BadNpcId { id })?,
        }
    }

    pub async fn save(&self, npc: Npc) -> Result<()> {
        self.tree.insert(&npc.id, &npc.data).await.erase_err()?;
        Ok(())
    }
}
