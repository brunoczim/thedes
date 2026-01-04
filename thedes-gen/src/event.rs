use std::array;

use rand::Rng;
use rand_distr::{Distribution, weighted::WeightedIndex};
use thedes_domain::{
    event::Event,
    game::Game,
    geometry::Coord,
    monster::{self, MonsterPosition},
};
use thedes_geometry::{
    orientation::Direction,
    rect::{InvalidRectDistr, UniformRectDistr},
};
use thiserror::Error;

use crate::random::ProabilityWeight;

#[derive(Debug, Error)]
pub enum DistrError {
    #[error("Invalid map rectangle")]
    InvalidMapRect(
        #[from]
        #[source]
        InvalidRectDistr,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventType {
    TrySpawnMonster,
    VanishMonster,
    TryMoveMonster,
    MonsterAttack,
}

impl EventType {
    pub const COUNT: usize = 4;

    pub const ALL: [Self; Self::COUNT] = [
        Self::TrySpawnMonster,
        Self::VanishMonster,
        Self::TryMoveMonster,
        Self::MonsterAttack,
    ];
}

#[derive(Debug, Clone)]
pub struct EventTypeDistr {
    cumulative_weights: [ProabilityWeight; EventType::COUNT],
}

impl EventTypeDistr {
    pub fn new<F>(mut density_function: F) -> Self
    where
        F: FnMut(EventType) -> ProabilityWeight,
    {
        let mut accumuled_weight = 0;
        let cumulative_weights = array::from_fn(|i| {
            accumuled_weight += density_function(EventType::ALL[i]);
            accumuled_weight
        });
        Self { cumulative_weights }
    }

    pub fn from_monster_count(x: Coord) -> Self {
        let cut = 10000;
        let x = x as ProabilityWeight;
        Self::new(|ty| {
            let weight = match ty {
                EventType::TrySpawnMonster => {
                    if x == 0 {
                        1
                    } else if x < cut {
                        cut * 2 - x
                    } else {
                        x / cut
                    }
                },
                EventType::VanishMonster => {
                    if x == 0 {
                        0
                    } else if x < cut {
                        x
                    } else {
                        x - cut
                    }
                },
                EventType::TryMoveMonster => x * cut / 10,
                EventType::MonsterAttack => x * cut / 10,
            };
            weight
        })
    }
}

impl Distribution<EventType> for EventTypeDistr {
    fn sample<R>(&self, rng: &mut R) -> EventType
    where
        R: Rng + ?Sized,
    {
        let last_cumulative_weight =
            self.cumulative_weights[self.cumulative_weights.len() - 1];
        let sampled_weight = rng.random_range(0 .. last_cumulative_weight);
        for (i, cumulative_weight) in
            self.cumulative_weights.into_iter().enumerate()
        {
            if sampled_weight < cumulative_weight {
                return EventType::ALL[i];
            }
        }
        panic!("sampled weight {sampled_weight} is out of requested bounds")
    }
}

#[derive(Debug, Clone)]
pub struct EventDistr<'a> {
    monsters: &'a monster::Registry,
    event_type_distr: EventTypeDistr,
    map_rect_uniforrm_distr: UniformRectDistr<Coord>,
}

impl<'a> EventDistr<'a> {
    pub fn new(game: &'a Game) -> Result<Self, DistrError> {
        let monsters = game.monster_registry();
        let monster_count = monsters.len() as Coord;
        let event_type_distr =
            EventTypeDistr::from_monster_count(monster_count);
        let map_rect_uniforrm_distr = UniformRectDistr::new(game.map().rect())?;
        Ok(Self { monsters, event_type_distr, map_rect_uniforrm_distr })
    }
}

impl<'a> Distribution<Event> for EventDistr<'a> {
    fn sample<R>(&self, rng: &mut R) -> Event
    where
        R: Rng + ?Sized,
    {
        match self.event_type_distr.sample(rng) {
            EventType::TrySpawnMonster => {
                let body = self.map_rect_uniforrm_distr.sample(rng);
                let facing = rng.random();
                Event::TrySpawnMonster(MonsterPosition::new(body, facing))
            },
            EventType::VanishMonster => {
                let index = rng.random_range(.. self.monsters.len());
                let (id, _) = self
                    .monsters
                    .get_by_index_as(index)
                    .expect("inconsistent indexing");
                Event::VanishMonster(id)
            },
            EventType::TryMoveMonster => {
                let index = rng.random_range(.. self.monsters.len());
                let (id, monster) = self
                    .monsters
                    .get_by_index_as(index)
                    .expect("inconsistent indexing");
                let curr_direction = monster.position().facing();
                let directions = Direction::ALL;
                let weights = directions.map(|direction| {
                    if direction == curr_direction { 5 } else { 1 }
                });
                let weighted = WeightedIndex::new(&weights)
                    .expect("no weight should be zero, no overflow");
                let direction = directions[weighted.sample(rng)];
                Event::TryMoveMonster(id, direction)
            },
            EventType::MonsterAttack => {
                let index = rng.random_range(.. self.monsters.len());
                let (id, _) = self
                    .monsters
                    .get_by_index_as(index)
                    .expect("inconsistent indexing");
                Event::MonsterAttack(id)
            },
        }
    }
}
