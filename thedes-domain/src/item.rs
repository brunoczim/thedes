use thiserror::Error;

use crate::bitpack::BitPack;

pub type StackableItem8Bits = u8;
pub type StackableEntry8Bits = u8;
pub type ItemBits = u8;
pub type SlotEntryBits = u8;

#[derive(Debug, Error)]
pub enum InvalidCount {
    #[error("Expected at most a count of {min}, found {found}")]
    TooLow { min: u8, found: u8 },

    #[error("Expected at most a count of {max}, found {found}")]
    TooHigh { max: u8, found: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Inventory {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SlotEntry {
    Vaccant,
    Stackable8(StackableEntry8),
}

impl From<StackableEntry8> for SlotEntry {
    fn from(entry: StackableEntry8) -> Self {
        Self::Stackable8(entry)
    }
}

impl SlotEntry {
    const VACCANT_BITS: SlotEntryBits = 0;
    const STACKABLE_8_OFFSET: SlotEntryBits = 1 + Self::VACCANT_BITS;
    const MAX_BITS: SlotEntryBits =
        Self::STACKABLE_8_OFFSET + StackableEntry8::MAX_BITS;
}

impl BitPack for SlotEntry {
    type BitVector = SlotEntryBits;
    const BIT_COUNT: u32 = 4;
    const ELEM_COUNT: usize = Self::MAX_BITS as usize + 1;

    fn pack(self) -> Self::BitVector {
        match self {
            Self::Vaccant => Self::VACCANT_BITS,
            Self::Stackable8(item) => item.pack() + Self::STACKABLE_8_OFFSET,
        }
    }

    fn unpack(bits: Self::BitVector) -> Option<Self> {
        if bits < Self::VACCANT_BITS {
            Some(Self::Vaccant)
        } else {
            StackableEntry8::unpack(bits - Self::STACKABLE_8_OFFSET)
                .map(Self::from)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StackableEntry8 {
    item: StackableItem8,
    count: u8,
}

impl StackableEntry8 {
    const MAX_BITS: StackableItem8Bits = (StackableItem8::MAX_BITS + 1) * 8 - 1;

    pub fn new(item: StackableItem8, count: u8) -> Result<Self, InvalidCount> {
        if count > 8 {
            Err(InvalidCount::TooHigh { max: 8, found: count })?
        }
        if count < 1 {
            Err(InvalidCount::TooLow { min: 1, found: count })?
        }
        Ok(Self { item, count })
    }

    pub fn item(self) -> StackableItem8 {
        self.item
    }

    pub fn count(self) -> u8 {
        self.count
    }
}

impl BitPack for StackableEntry8 {
    type BitVector = StackableEntry8Bits;
    const BIT_COUNT: u32 = 3;
    const ELEM_COUNT: usize = Self::MAX_BITS as usize + 1;

    fn pack(self) -> Self::BitVector {
        self.count - 1
    }

    fn unpack(bits: Self::BitVector) -> Option<Self> {
        Self::new(StackableItem8::Stick, bits + 1).ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Item {
    Stackable8(StackableItem8),
}

impl From<StackableItem8> for Item {
    fn from(item: StackableItem8) -> Self {
        Self::Stackable8(item)
    }
}

impl Item {
    const STACKABLE_8_OFFSET: ItemBits = 0;
    const MAX_BITS: ItemBits =
        Self::STACKABLE_8_OFFSET + StackableItem8::MAX_BITS;
}

impl BitPack for Item {
    type BitVector = ItemBits;
    const BIT_COUNT: u32 = 1;
    const ELEM_COUNT: usize = Self::MAX_BITS as usize + 1;

    fn pack(self) -> Self::BitVector {
        match self {
            Self::Stackable8(item) => item.pack() + Self::STACKABLE_8_OFFSET,
        }
    }

    fn unpack(bits: Self::BitVector) -> Option<Self> {
        StackableItem8::unpack(bits - Self::STACKABLE_8_OFFSET).map(Self::from)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum StackableItem8 {
    Stick,
}

impl StackableItem8 {
    const STICK_BITS: StackableItem8Bits = Self::Stick as StackableItem8Bits;
    const MAX_BITS: StackableItem8Bits = Self::STICK_BITS;
}

impl BitPack for StackableItem8 {
    type BitVector = StackableItem8Bits;
    const BIT_COUNT: u32 = 1;
    const ELEM_COUNT: usize = Self::MAX_BITS as usize + 1;

    fn pack(self) -> Self::BitVector {
        self as StackableItem8Bits
    }

    fn unpack(bits: Self::BitVector) -> Option<Self> {
        Some(match bits {
            Self::STICK_BITS => Self::Stick,
            _ => None?,
        })
    }
}
