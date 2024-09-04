use crate::bitpack::BitPack;

type BlockBits = u8;
type SpecialBlockBits = u8;
type PlaceableBlockBits = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Block {
    Placeable(PlaceableBlock),
    Special(SpecialBlock),
}

impl Block {
    const PLACEABLE_OFFSET: BlockBits = 0;
    const SPECIAL_OFFSET: BlockBits =
        1 + Self::PLACEABLE_OFFSET + PlaceableBlock::MAX_BITS;
    const MAX_BITS: BlockBits = Self::SPECIAL_OFFSET + SpecialBlock::MAX_BITS;

    pub fn placeable(self) -> Option<PlaceableBlock> {
        let Self::Placeable(block) = self else { None? };
        Some(block)
    }

    pub fn special(self) -> Option<SpecialBlock> {
        let Self::Special(block) = self else { None? };
        Some(block)
    }
}

impl BitPack for Block {
    type BitVector = BlockBits;
    const BIT_COUNT: u32 = 2;
    const ELEM_COUNT: usize = Self::MAX_BITS as usize + 1;

    fn pack(self) -> Self::BitVector {
        match self {
            Self::Placeable(block) => Self::PLACEABLE_OFFSET + block.pack(),
            Self::Special(block) => Self::SPECIAL_OFFSET + block.pack(),
        }
    }

    fn unpack(bits: Self::BitVector) -> Option<Self> {
        if bits < Self::SPECIAL_OFFSET {
            PlaceableBlock::unpack(bits).map(Self::Placeable)
        } else {
            SpecialBlock::unpack(bits - Self::SPECIAL_OFFSET).map(Self::Special)
        }
    }
}

impl From<PlaceableBlock> for Block {
    fn from(block: PlaceableBlock) -> Self {
        Self::Placeable(block)
    }
}

impl From<SpecialBlock> for Block {
    fn from(block: SpecialBlock) -> Self {
        Self::Special(block)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlaceableBlock {
    Air = 0,
    Stick = 1,
}

impl PlaceableBlock {
    const AIR_BITS: PlaceableBlockBits = Self::Air as PlaceableBlockBits;
    const STICK_BITS: PlaceableBlockBits = Self::Stick as PlaceableBlockBits;
    const MAX_BITS: PlaceableBlockBits = Self::STICK_BITS;

    pub const ALL: [Self; 2] = [Self::Air, Self::Stick];
}

impl BitPack for PlaceableBlock {
    type BitVector = PlaceableBlockBits;
    const BIT_COUNT: u32 = 1;
    const ELEM_COUNT: usize = Self::MAX_BITS as usize + 1;

    fn pack(self) -> Self::BitVector {
        self as PlaceableBlockBits
    }

    fn unpack(bits: Self::BitVector) -> Option<Self> {
        Some(match bits {
            Self::AIR_BITS => Self::Air,
            Self::STICK_BITS => Self::Stick,
            _ => None?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SpecialBlock {
    Player = 0,
}

impl SpecialBlock {
    const PLAYER_BITS: SpecialBlockBits = Self::Player as SpecialBlockBits;
    const MAX_BITS: SpecialBlockBits = Self::PLAYER_BITS;

    pub const ALL: [Self; 1] = [Self::Player];
}

impl BitPack for SpecialBlock {
    type BitVector = SpecialBlockBits;
    const BIT_COUNT: u32 = 1;
    const ELEM_COUNT: usize = Self::MAX_BITS as usize + 1;

    fn pack(self) -> Self::BitVector {
        self as SpecialBlockBits
    }

    fn unpack(bits: Self::BitVector) -> Option<Self> {
        Some(match bits {
            Self::PLAYER_BITS => Self::Player,
            _ => None?,
        })
    }
}
