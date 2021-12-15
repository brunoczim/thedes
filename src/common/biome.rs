use super::ground::Ground;
use std::fmt;

/// A biome type.
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
    /// This biome is a plain.
    Plain,
    /// This biome is a sand desert.
    Desert,
    /// This biome is a rock desert.
    RockDesert,
}

impl fmt::Display for Biome {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(match self {
            Biome::Plain => "plain",
            Biome::Desert => "desert",
            Biome::RockDesert => "rocks",
        })
    }
}

impl Biome {
    /// Returns the main ground type of this biome.
    pub fn main_ground(&self) -> Ground {
        match self {
            Biome::Plain => Ground::Grass,
            Biome::Desert => Ground::Sand,
            Biome::RockDesert => Ground::Rock,
        }
    }
}
