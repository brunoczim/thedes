use crate::{
    human,
    map::{Coord, Map},
    random::make_rng,
};
use gardiz::{coord::Vec2, direc::Direction};
use kopidaz::tree::Tree;
use rand::{rngs::StdRng, Rng};
use thedes_common::{
    block::Block,
    error::{BadPlayerId, Error},
    seed::Seed,
    Result,
    ResultExt,
};

pub use thedes_common::player::{Data, Id, Player, MAX_HEALTH};

pub async fn move_around(
    player: &mut Player,
    direc: Direction,
    map: &mut Map,
    registry: &Registry,
) -> Result<()> {
    let block = player.block();
    human::move_around(&mut player.data.body, block, direc, map).await?;
    registry.save(*player).await?;
    Ok(())
}

pub async fn step(
    player: &mut Player,
    direc: Direction,
    map: &mut Map,
    registry: &Registry,
) -> Result<()> {
    let block = player.block();
    human::step(&mut player.data.body, block, direc, map).await?;
    registry.save(*player).await?;
    Ok(())
}

pub async fn turn_around(
    player: &mut Player,
    direc: Direction,
    map: &mut Map,
    registry: &Registry,
) -> Result<()> {
    let block = player.block();
    human::turn_around(&mut player.data.body, block, direc, map).await?;
    registry.save(*player).await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Registry {
    tree: Tree<Id, Data>,
}

impl Registry {
    pub async fn new(db: &sled::Db) -> Result<Self> {
        let tree = Tree::open(db, "player::Registry").await.erase_err()?;
        Ok(Self { tree })
    }

    pub async fn register(
        &self,
        db: &sled::Db,
        seed: Seed,
        map: &mut Map,
    ) -> Result<Player> {
        let mut rng = make_rng::<_, StdRng>(seed, 0u128);

        let low = Coord::max_value() / 5 * 2;
        let high = Coord::max_value() / 5 * 3 + Coord::max_value() % 5;
        let mut body = human::Body {
            head: Vec2 {
                x: rng.gen_range(low .. high),
                y: rng.gen_range(low .. high),
            },
            facing: Direction::Up,
        };

        while map.entry(body.head).await?.block != Block::Empty
            || map.entry(body.pointer()).await?.block != Block::Empty
        {
            body.head = Vec2 {
                x: rng.gen_range(low .. high),
                y: rng.gen_range(low .. high),
            };
        }

        let (id, data) = self
            .tree
            .id_builder()
            .id_maker(|bits| Id(bits as _))
            .data_maker(|_| Data {
                body,
                health: MAX_HEALTH,
                max_health: MAX_HEALTH,
            })
            .error_conversor(Error::erase)
            .generate(db)
            .await?;

        human::write_on_map(data.body, Block::Player(id), map).await?;
        Ok(Player { id, data })
    }

    pub async fn load(&self, id: Id) -> Result<Player> {
        match self.tree.get(&id).await.erase_err()? {
            Some(data) => Ok(Player { id, data }),
            None => Err(BadPlayerId { id })?,
        }
    }

    pub async fn save(&self, player: Player) -> Result<()> {
        self.tree.insert(&player.id, &player.data).await.erase_err()?;
        Ok(())
    }
}
