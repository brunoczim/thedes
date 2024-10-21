use std::fmt;

use crate::bitpack::BitPack;

type GroundBits = u8;

type BiomeBits = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Ground {
    Grass = 0,
    Sand = 1,
    Stone = 2,
}

impl Ground {
    const GRASS_BITS: GroundBits = Self::Grass as GroundBits;
    const SAND_BITS: GroundBits = Self::Sand as GroundBits;
    const STONE_BITS: GroundBits = Self::Stone as GroundBits;
    const MAX_BITS: GroundBits = Self::STONE_BITS;

    pub const ALL: [Self; 3] = [Self::Grass, Self::Sand, Self::Stone];
}

impl Default for Ground {
    fn default() -> Self {
        Self::Grass
    }
}

impl BitPack for Ground {
    type BitVector = u8;
    const BIT_COUNT: u32 = 2;
    const ELEM_COUNT: usize = Self::MAX_BITS as usize + 1;

    fn pack(self) -> Self::BitVector {
        self as GroundBits
    }

    fn unpack(bits: Self::BitVector) -> Option<Self> {
        Some(match bits {
            Self::GRASS_BITS => Self::Grass,
            Self::SAND_BITS => Self::Sand,
            Self::STONE_BITS => Self::Stone,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Biome {
    Plains = 0,
    Desert = 1,
    Wasteland = 2,
}

impl Biome {
    const PLAINS_BITS: BiomeBits = Self::Plains as BiomeBits;
    const DESERT_BITS: BiomeBits = Self::Desert as BiomeBits;
    const WASTELAND_BITS: BiomeBits = Self::Wasteland as BiomeBits;
    const MAX_BITS: BiomeBits = Self::WASTELAND_BITS;

    pub const ALL: [Self; 3] = [Self::Plains, Self::Desert, Self::Wasteland];
}

impl Default for Biome {
    fn default() -> Self {
        Self::Plains
    }
}

impl BitPack for Biome {
    type BitVector = u8;
    const BIT_COUNT: u32 = 2;
    const ELEM_COUNT: usize = Self::MAX_BITS as usize + 1;

    fn pack(self) -> Self::BitVector {
        self as BiomeBits
    }

    fn unpack(bits: Self::BitVector) -> Option<Self> {
        Some(match bits {
            Self::PLAINS_BITS => Self::Plains,
            Self::DESERT_BITS => Self::Desert,
            Self::WASTELAND_BITS => Self::Wasteland,
            _ => return None,
        })
    }
}

impl fmt::Display for Biome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Plains => "plains",
            Self::Desert => "desert",
            Self::Wasteland => "wasteland",
        })
    }
}
