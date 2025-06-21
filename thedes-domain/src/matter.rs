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
    Default,
    Serialize,
    Deserialize,
)]
pub enum Biome {
    #[default]
    Plains,
    Desert,
    Wasteland,
}

impl Biome {
    pub const COUNT: usize = 3;

    pub const ALL: [Self; Self::COUNT] =
        [Self::Plains, Self::Desert, Self::Wasteland];
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
    Default,
    Serialize,
    Deserialize,
)]
pub enum Ground {
    #[default]
    Grass,
    Sand,
    Stone,
}

impl Ground {
    pub const COUNT: usize = 3;

    pub const ALL: [Self; Self::COUNT] = [Self::Grass, Self::Sand, Self::Stone];
}
