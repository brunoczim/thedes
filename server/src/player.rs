use gardiz::{coord::Vec2, direc::Direction};
use kopidaz::tree::Tree;
use rand::{rngs::StdRng, Rng};
use std::{error::Error, fmt};
use thedes_common::{
    block::Block,
    health::Health,
    map::Coord,
    player::Player,
    seed::Seed,
    Result,
    ResultExt,
};

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
pub async fn step(&mut self, direc: Direction, game: &SavedGame) -> Result<()> {
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
