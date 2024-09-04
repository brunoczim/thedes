use crate::bitpack::BitPack;

type GroundBits = u8;

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
