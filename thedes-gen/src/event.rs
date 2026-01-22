use std::array;

use rand::Rng;
use rand_distr::{
    Distribution,
    Triangular,
    TriangularError,
    weighted::WeightedIndex,
};
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
    #[error("Failed to create distribution for monster follow limit")]
    InvalidMonsterFollowLimitDistr(#[source] TriangularError),
    #[error("Failed to create distribution for monster follow period")]
    InvalidMonsterFollowPeriodDistr(#[source] TriangularError),
}

#[derive(Debug, Error)]
pub enum InvalidMonsterFollowLimit {
    #[error(
        "Monster-follow limit must be in the interval [{}, {}], given {}",
        DistrConfig::MIN_FOLLOW_LIMIT,
        DistrConfig::MAX_FOLLOW_LIMIT,
        _0
    )]
    Range(u32),
    #[error(
        "Peak monster follow limit {peak} must be between min and max {min} \
         and {max}"
    )]
    PeakOutOfBounds { min: u32, max: u32, peak: u32 },
    #[error(
        "Minimum monster follow limit {min} cannot be greater than maximum \
         {max}"
    )]
    BoundOrder { min: u32, max: u32 },
}

#[derive(Debug, Error)]
pub enum InvalidMonsterFollowPeriod {
    #[error(
        "Monster-follow period must be in the interval [{}, {}], given {}",
        DistrConfig::MIN_FOLLOW_PERIOD,
        DistrConfig::MAX_FOLLOW_PERIOD,
        _0
    )]
    Range(Coord),
    #[error(
        "Peak monster follow period {peak} must be between min and max {min} \
         and {max}"
    )]
    PeakOutOfBounds { min: Coord, max: Coord, peak: Coord },
    #[error(
        "Minimum monster follow period {min} cannot be greater than maximum \
         {max}"
    )]
    BoundOrder { min: Coord, max: Coord },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventType {
    TrySpawnMonster,
    VanishMonster,
    TryMoveMonster,
    MonsterAttack,
    FollowPlayer,
}

impl EventType {
    pub const COUNT: usize = 5;

    pub const ALL: [Self; Self::COUNT] = [
        Self::TrySpawnMonster,
        Self::VanishMonster,
        Self::TryMoveMonster,
        Self::MonsterAttack,
        Self::FollowPlayer,
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
                EventType::TryMoveMonster => x * cut / 100,
                EventType::MonsterAttack => x * cut / 5,
                EventType::FollowPlayer => x,
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
pub struct DistrConfig {
    monster_follow_limit_min: u32,
    monster_follow_limit_peak: u32,
    monster_follow_limit_max: u32,
    monster_follow_period_min: Coord,
    monster_follow_period_peak: Coord,
    monster_follow_period_max: Coord,
}

impl DistrConfig {
    pub const MIN_FOLLOW_PERIOD: Coord = 1;
    pub const MAX_FOLLOW_PERIOD: Coord = Coord::MAX;

    pub const MIN_FOLLOW_LIMIT: u32 = 1;
    pub const MAX_FOLLOW_LIMIT: u32 = u32::MAX;

    pub fn new() -> Self {
        Self {
            monster_follow_period_min: 1500,
            monster_follow_period_peak: 2000,
            monster_follow_period_max: 2500,
            monster_follow_limit_min: 100,
            monster_follow_limit_peak: 1000,
            monster_follow_limit_max: 5000,
        }
    }

    pub fn monster_follow_limit_min(&self) -> u32 {
        self.monster_follow_limit_min
    }

    pub fn set_monster_follow_limit_min(
        &mut self,
        value: u32,
    ) -> Result<(), InvalidMonsterFollowLimit> {
        if value < Self::MIN_FOLLOW_LIMIT || value > Self::MAX_FOLLOW_LIMIT {
            Err(InvalidMonsterFollowLimit::Range(value))?;
        }
        if self.monster_follow_limit_peak() < value {
            Err(InvalidMonsterFollowLimit::BoundOrder {
                min: value,
                max: self.monster_follow_limit_max(),
            })?;
        }
        if self.monster_follow_limit_max() <= value {
            Err(InvalidMonsterFollowLimit::PeakOutOfBounds {
                min: value,
                max: self.monster_follow_limit_max(),
                peak: self.monster_follow_limit_peak(),
            })?;
        }

        self.monster_follow_limit_min = value;
        Ok(())
    }

    pub fn with_monster_follow_limit_min(
        mut self,
        value: u32,
    ) -> Result<Self, InvalidMonsterFollowLimit> {
        self.set_monster_follow_limit_min(value)?;
        Ok(self)
    }

    pub fn monster_follow_limit_peak(&self) -> u32 {
        self.monster_follow_limit_peak
    }

    pub fn set_monster_follow_limit_peak(
        &mut self,
        value: u32,
    ) -> Result<(), InvalidMonsterFollowLimit> {
        if self.monster_follow_limit_max() < value
            || self.monster_follow_limit_min() > value
        {
            Err(InvalidMonsterFollowLimit::PeakOutOfBounds {
                min: value,
                max: self.monster_follow_limit_max(),
                peak: self.monster_follow_limit_peak(),
            })?;
        }

        self.monster_follow_limit_peak = value;
        Ok(())
    }

    pub fn with_monster_follow_limit_peak(
        mut self,
        value: u32,
    ) -> Result<Self, InvalidMonsterFollowLimit> {
        self.set_monster_follow_limit_peak(value)?;
        Ok(self)
    }

    pub fn monster_follow_limit_max(&self) -> u32 {
        self.monster_follow_limit_max
    }

    pub fn set_monster_follow_limit_max(
        &mut self,
        value: u32,
    ) -> Result<(), InvalidMonsterFollowLimit> {
        if value < Self::MIN_FOLLOW_LIMIT || value > Self::MAX_FOLLOW_LIMIT {
            Err(InvalidMonsterFollowLimit::Range(value))?;
        }
        if self.monster_follow_limit_peak() > value {
            Err(InvalidMonsterFollowLimit::BoundOrder {
                min: self.monster_follow_limit_min(),
                max: value,
            })?;
        }
        if self.monster_follow_limit_min() >= value {
            Err(InvalidMonsterFollowLimit::PeakOutOfBounds {
                min: self.monster_follow_limit_min(),
                max: value,
                peak: self.monster_follow_limit_peak(),
            })?;
        }

        self.monster_follow_limit_max = value;

        Ok(())
    }

    pub fn with_monster_follow_limit_max(
        mut self,
        value: u32,
    ) -> Result<Self, InvalidMonsterFollowLimit> {
        self.set_monster_follow_limit_max(value)?;
        Ok(self)
    }

    pub fn monster_follow_period_min(&self) -> Coord {
        self.monster_follow_period_min
    }

    pub fn set_monster_follow_period_min(
        &mut self,
        value: Coord,
    ) -> Result<(), InvalidMonsterFollowPeriod> {
        if value < Self::MIN_FOLLOW_PERIOD || value > Self::MAX_FOLLOW_PERIOD {
            Err(InvalidMonsterFollowPeriod::Range(value))?;
        }
        if self.monster_follow_period_peak() < value {
            Err(InvalidMonsterFollowPeriod::BoundOrder {
                min: value,
                max: self.monster_follow_period_max(),
            })?;
        }
        if self.monster_follow_period_max() <= value {
            Err(InvalidMonsterFollowPeriod::PeakOutOfBounds {
                min: value,
                max: self.monster_follow_period_max(),
                peak: self.monster_follow_period_peak(),
            })?;
        }

        self.monster_follow_period_min = value;
        Ok(())
    }

    pub fn with_monster_follow_period_min(
        mut self,
        value: Coord,
    ) -> Result<Self, InvalidMonsterFollowPeriod> {
        self.set_monster_follow_period_min(value)?;
        Ok(self)
    }

    pub fn monster_follow_period_peak(&self) -> Coord {
        self.monster_follow_period_peak
    }

    pub fn set_monster_follow_period_peak(
        &mut self,
        value: Coord,
    ) -> Result<(), InvalidMonsterFollowPeriod> {
        if self.monster_follow_period_max() < value
            || self.monster_follow_period_min() > value
        {
            Err(InvalidMonsterFollowPeriod::PeakOutOfBounds {
                min: value,
                max: self.monster_follow_period_max(),
                peak: self.monster_follow_period_peak(),
            })?;
        }

        self.monster_follow_period_peak = value;
        Ok(())
    }

    pub fn with_monster_follow_period_peak(
        mut self,
        value: Coord,
    ) -> Result<Self, InvalidMonsterFollowPeriod> {
        self.set_monster_follow_period_peak(value)?;
        Ok(self)
    }

    pub fn monster_follow_period_max(&self) -> Coord {
        self.monster_follow_period_max
    }

    pub fn set_monster_follow_period_max(
        &mut self,
        value: Coord,
    ) -> Result<(), InvalidMonsterFollowPeriod> {
        if value < Self::MIN_FOLLOW_PERIOD || value > Self::MAX_FOLLOW_PERIOD {
            Err(InvalidMonsterFollowPeriod::Range(value))?;
        }
        if self.monster_follow_period_peak() > value {
            Err(InvalidMonsterFollowPeriod::BoundOrder {
                min: self.monster_follow_period_min(),
                max: value,
            })?;
        }
        if self.monster_follow_period_min() >= value {
            Err(InvalidMonsterFollowPeriod::PeakOutOfBounds {
                min: self.monster_follow_period_min(),
                max: value,
                peak: self.monster_follow_period_peak(),
            })?;
        }

        self.monster_follow_period_max = value;

        Ok(())
    }

    pub fn with_monster_follow_period_max(
        mut self,
        value: Coord,
    ) -> Result<Self, InvalidMonsterFollowPeriod> {
        self.set_monster_follow_period_max(value)?;
        Ok(self)
    }

    pub fn finish<'a>(
        &self,
        game: &'a Game,
    ) -> Result<EventDistr<'a>, DistrError> {
        let monsters = game.monster_registry();
        let monster_count = monsters.len() as Coord;
        let event_type_distr =
            EventTypeDistr::from_monster_count(monster_count);
        let map_rect_uniform_distr = UniformRectDistr::new(game.map().rect())?;

        let monster_follow_limit_distr = Triangular::new(
            self.monster_follow_limit_min as f64,
            self.monster_follow_limit_max as f64,
            self.monster_follow_limit_peak as f64,
        )
        .map_err(DistrError::InvalidMonsterFollowLimitDistr)?;

        let monster_follow_period_distr = Triangular::new(
            f64::from(self.monster_follow_period_min),
            f64::from(self.monster_follow_period_max),
            f64::from(self.monster_follow_period_peak),
        )
        .map_err(DistrError::InvalidMonsterFollowPeriodDistr)?;

        Ok(EventDistr {
            monsters,
            event_type_distr,
            map_rect_uniform_distr,
            monster_follow_limit_distr,
            monster_follow_period_distr,
        })
    }
}

#[derive(Debug, Clone)]
pub struct EventDistr<'a> {
    monsters: &'a monster::Registry,
    event_type_distr: EventTypeDistr,
    map_rect_uniform_distr: UniformRectDistr<Coord>,
    monster_follow_limit_distr: Triangular<f64>,
    monster_follow_period_distr: Triangular<f64>,
}

impl<'a> Distribution<Event> for EventDistr<'a> {
    fn sample<R>(&self, rng: &mut R) -> Event
    where
        R: Rng + ?Sized,
    {
        match self.event_type_distr.sample(rng) {
            EventType::TrySpawnMonster => {
                let body = self.map_rect_uniform_distr.sample(rng);
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
            EventType::FollowPlayer => {
                let index = rng.random_range(.. self.monsters.len());
                let (id, _) = self
                    .monsters
                    .get_by_index_as(index)
                    .expect("inconsistent indexing");
                let limit = self.monster_follow_limit_distr.sample(rng) as u32;
                let period =
                    self.monster_follow_period_distr.sample(rng) as Coord;
                Event::FollowPlayer { id, period, limit }
            },
        }
    }
}
