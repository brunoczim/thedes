use num::FromPrimitive;
use num_derive::FromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Season {
    Ware,
    Summer,
    Harvest,
    Winter,
}

impl Default for Season {
    fn default() -> Self {
        Self::Ware
    }
}

impl Season {
    pub const YEAR_DURATION: u64 = Self::Ware.duration()
        + Self::Summer.duration()
        + Self::Harvest.duration()
        + Self::Winter.duration();

    pub const fn duration(self) -> u64 {
        match self {
            Self::Ware | Self::Harvest => 14,
            Self::Summer | Self::Winter => 13,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive,
)]
#[repr(u64)]
pub enum LunarPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningCrescent,
    LastQuarter,
    WaningGibbous,
}

impl Default for LunarPhase {
    fn default() -> Self {
        Self::New
    }
}

impl LunarPhase {
    const COUNT: u64 = 8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    stamp: u64,
}

impl Time {
    pub const MAX_DAYS: u64 = 270;

    const CIRCADIAN_CYCLE_SIZE: u64 = 1 << 16;

    const MAX_STAMP: u64 = Self::MAX_DAYS * Self::CIRCADIAN_CYCLE_SIZE;

    pub fn new() -> Self {
        Self { stamp: 0 }
    }

    pub fn on_tick(&mut self) {
        self.stamp = (self.stamp + 1).min(Self::MAX_STAMP);
    }

    pub fn world_ended(&self) -> bool {
        self.stamp >= Self::MAX_STAMP
    }

    pub fn day(&self) -> u64 {
        self.stamp / Self::CIRCADIAN_CYCLE_SIZE
    }

    pub fn day_of_year(&self) -> u64 {
        self.day() % Season::YEAR_DURATION
    }

    pub fn season(&self) -> Season {
        let mut day_of_year = self.day_of_year();
        if day_of_year < Season::Ware.duration() {
            return Season::Ware;
        }
        day_of_year -= Season::Ware.duration();
        if day_of_year < Season::Summer.duration() {
            return Season::Summer;
        }
        day_of_year -= Season::Summer.duration();
        if day_of_year < Season::Harvest.duration() {
            return Season::Harvest;
        }
        Season::Winter
    }

    pub fn lunar_phase(&self) -> LunarPhase {
        LunarPhase::from_u64(self.day() % LunarPhase::COUNT).unwrap_or_default()
    }
}
