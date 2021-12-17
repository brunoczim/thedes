use crate::ground::Ground;
use std::fmt;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Biome {
    Unknown,
    Plain,
    Desert,
    RockDesert,
}

impl fmt::Display for Biome {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(match self {
            Biome::Unknown => "unknown",
            Biome::Plain => "plain",
            Biome::Desert => "desert",
            Biome::RockDesert => "rocks",
        })
    }
}

impl Biome {
    #[inline]
    pub fn main_ground(&self) -> Ground {
        match self {
            Biome::Unknown => Ground::Unknown,
            Biome::Plain => Ground::Grass,
            Biome::Desert => Ground::Sand,
            Biome::RockDesert => Ground::Rock,
        }
    }
}
