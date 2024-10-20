use std::fmt;

use num::{rational::Ratio, FromPrimitive};
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum CircadianCycleStep {
    Sunrise,
    DayLight,
    Sunset,
    Night,
}

impl CircadianCycleStep {
    pub const fn next(self) -> Self {
        match self {
            Self::Sunrise => Self::DayLight,
            Self::DayLight => Self::Sunset,
            Self::Sunset => Self::Night,
            Self::Night => Self::Sunrise,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Sunrise => Self::Night,
            Self::DayLight => Self::Sunrise,
            Self::Sunset => Self::DayLight,
            Self::Night => Self::Sunset,
        }
    }

    const fn starting_duration(self, season: Season) -> u64 {
        match (self, season) {
            // 3h30
            (Self::Sunrise, Season::Ware) => 7,
            // 8h30
            (Self::DayLight, Season::Ware) => 17,
            // 4h
            (Self::Sunset, Season::Ware) => 8,
            // 8h
            (Self::Night, Season::Ware) => 16,

            // 2h
            (Self::Sunrise, Season::Summer) => 4,
            // 11h30
            (Self::DayLight, Season::Summer) => 23,
            // 5h
            (Self::Sunset, Season::Summer) => 10,
            // 5h30
            (Self::Night, Season::Summer) => 11,

            // 4h
            (Self::Sunrise, Season::Harvest) => 8,
            // 8h
            (Self::DayLight, Season::Harvest) => 16,
            // 3h30
            (Self::Sunset, Season::Harvest) => 7,
            // 8h30
            (Self::Night, Season::Harvest) => 17,

            // 4h30
            (Self::Sunrise, Season::Winter) => 9,
            // 5h
            (Self::DayLight, Season::Winter) => 10,
            // 2h
            (Self::Sunset, Season::Winter) => 4,
            // 12h30
            (Self::Night, Season::Winter) => 25,
        }
    }

    const fn duration_in_day_of_year(self, day_of_year: u64) -> u64 {
        let season = Season::for_day_of_year(day_of_year);
        let season_duration = season.duration();
        let next_season = season.next();
        let starting = self.starting_duration(season);
        let ending = self.starting_duration(next_season);
        let day_of_season =
            day_of_year + season.duration() - season.year_cumulative_duration();
        (starting * (season_duration - day_of_season) + ending * day_of_season)
            / season_duration
    }

    const fn cumul_dur_in_day_of_year(self, day_of_year: u64) -> u64 {
        match self {
            Self::Sunrise => self.duration_in_day_of_year(day_of_year),
            _ => {
                self.prev().cumul_dur_in_day_of_year(day_of_year)
                    + self.duration_in_day_of_year(day_of_year)
            },
        }
    }

    pub const fn for_day_of_year(
        day_of_year: u64,
        clock_day_ratio: Ratio<u64>,
    ) -> Self {
        let clock_step = *clock_day_ratio.numer();
        let clock_total = *clock_day_ratio.denom();

        let total_weights = Self::Night.cumul_dur_in_day_of_year(day_of_year);
        if clock_step * total_weights
            < Self::Sunrise.cumul_dur_in_day_of_year(day_of_year) * clock_total
        {
            Self::Sunrise
        } else if clock_step * total_weights
            < Self::DayLight.cumul_dur_in_day_of_year(day_of_year) * clock_total
        {
            Self::DayLight
        } else if clock_step * total_weights
            < Self::Sunset.cumul_dur_in_day_of_year(day_of_year) * clock_total
        {
            Self::Sunset
        } else {
            Self::Night
        }
    }

    const fn total_for_day_of_year(day_of_year: u64) -> u64 {
        Self::Night.cumul_dur_in_day_of_year(day_of_year)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
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
    pub const ALL: [Self; 4] =
        [Self::Ware, Self::Summer, Self::Harvest, Self::Winter];

    pub const YEAR_DURATION: u64 = Self::Winter.year_cumulative_duration();

    pub const fn duration(self) -> u64 {
        match self {
            Self::Ware | Self::Harvest => 14,
            Self::Summer | Self::Winter => 13,
        }
    }

    pub const fn year_cumulative_duration(self) -> u64 {
        match self {
            Self::Ware => self.duration(),
            _ => self.prev().year_cumulative_duration() + self.duration(),
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Ware => Self::Summer,
            Self::Summer => Self::Harvest,
            Self::Harvest => Self::Winter,
            Self::Winter => Self::Ware,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Ware => Self::Winter,
            Self::Summer => Self::Ware,
            Self::Harvest => Self::Summer,
            Self::Winter => Self::Harvest,
        }
    }

    pub const fn for_day_of_year(day_of_year: u64) -> Self {
        if day_of_year < Self::Ware.year_cumulative_duration() {
            Self::Ware
        } else if day_of_year < Self::Summer.year_cumulative_duration() {
            Self::Summer
        } else if day_of_year < Self::Harvest.year_cumulative_duration() {
            Self::Harvest
        } else {
            Self::Winter
        }
    }

    pub const fn into_str(self) -> &'static str {
        match self {
            Self::Ware => "ware",
            Self::Summer => "summer",
            Self::Harvest => "harvest",
            Self::Winter => "winter",
        }
    }
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.into_str())
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    FromPrimitive,
    Serialize,
    Deserialize,
)]
#[repr(u64)]
pub enum LunarPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
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

    pub fn set(&mut self, stamp: u64) {
        self.stamp = stamp.min(Self::MAX_STAMP);
    }

    pub fn set_in_day_clock(&mut self, clock: Ratio<u64>) {
        let day = self.day();
        let step = (clock * Self::CIRCADIAN_CYCLE_SIZE)
            .to_integer()
            .min(Self::CIRCADIAN_CYCLE_SIZE - 1);
        self.set(step + day * Self::CIRCADIAN_CYCLE_SIZE);
    }

    pub fn set_circadian_cycle_step(&mut self, step: CircadianCycleStep) {
        let day_of_year = self.day_of_year();
        let relative = step.cumul_dur_in_day_of_year(day_of_year)
            - step.duration_in_day_of_year(day_of_year);
        self.set_in_day_clock(Ratio::new(
            relative,
            CircadianCycleStep::total_for_day_of_year(day_of_year),
        ));
    }

    pub fn set_day(&mut self, day: u64) {
        let circadian_step = self.stamp % Self::CIRCADIAN_CYCLE_SIZE;
        self.set(circadian_step + day * Self::CIRCADIAN_CYCLE_SIZE);
    }

    pub fn set_season(&mut self, season: Season) {
        let year = self.year();
        let day = season.year_cumulative_duration() - season.duration();
        self.set_day(day + year * Season::YEAR_DURATION);
    }

    pub fn set_lunar_phase(&mut self, phase: LunarPhase) {
        let month = self.month();
        let phase = phase as u64;
        self.set_day(phase + month * LunarPhase::COUNT);
    }

    pub fn set_day_of_year(&mut self, day: u64) {
        let circadian_step = self.stamp % Self::CIRCADIAN_CYCLE_SIZE;
        let year = self.year();
        self.set(
            circadian_step
                + (day % Season::YEAR_DURATION) * Self::CIRCADIAN_CYCLE_SIZE
                + year * Season::YEAR_DURATION * Self::CIRCADIAN_CYCLE_SIZE,
        );
    }

    pub fn set_year(&mut self, year: u64) {
        let day_of_year = self.day_of_year();
        self.set_day(day_of_year + year * Season::YEAR_DURATION);
    }

    pub fn on_tick(&mut self) {
        self.set(self.stamp + 1);
    }

    pub fn world_ended(&self) -> bool {
        self.stamp >= Self::MAX_STAMP
    }

    pub fn in_day_clock(&self) -> Ratio<u64> {
        Ratio::new(
            self.stamp % Self::CIRCADIAN_CYCLE_SIZE,
            Self::CIRCADIAN_CYCLE_SIZE,
        )
    }

    pub fn circadian_cycle_step(&self) -> CircadianCycleStep {
        CircadianCycleStep::for_day_of_year(
            self.day_of_year(),
            self.in_day_clock(),
        )
    }

    pub fn day(&self) -> u64 {
        self.stamp / Self::CIRCADIAN_CYCLE_SIZE
    }

    pub fn day_of_year(&self) -> u64 {
        self.day() % Season::YEAR_DURATION
    }

    pub fn year(&self) -> u64 {
        self.day() / Season::YEAR_DURATION
    }

    pub fn season(&self) -> Season {
        let day_of_year = self.day_of_year();
        Season::for_day_of_year(day_of_year)
    }

    pub fn month(&self) -> u64 {
        self.day() / LunarPhase::COUNT
    }

    pub fn lunar_phase(&self) -> LunarPhase {
        LunarPhase::from_u64(self.day() % LunarPhase::COUNT).unwrap_or_default()
    }
}
