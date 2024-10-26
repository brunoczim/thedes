use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id {
    bits: u8,
}

impl Id {
    fn new_raw(bits: u8) -> Self {
        Self { bits }
    }

    pub fn new(bits: u8) -> Option<Self> {
        if bits == 0 {
            None
        } else {
            Some(Self::new_raw(bits))
        }
    }

    pub fn bits(self) -> u8 {
        self.bits
    }

    pub fn option_bits(this: Option<Self>) -> u8 {
        this.map_or(0, Self::bits)
    }
}

#[derive(Debug, Error)]
pub enum AllocError {
    #[error("Thede IDs are exhausted")]
    Exhausted,
}

#[derive(Debug, Error)]
pub enum DeallocError {
    #[error("Thede ID {} is already free", .0.bits())]
    AlreadyFree(Id),
}

type RegistryMask = u128;

#[derive(Debug, Clone)]
pub struct Registry {
    alloc_masks: [RegistryMask; Self::MASKS],
}

impl Registry {
    const MASKS: usize = 2;

    const MASK_BITS: u8 = RegistryMask::BITS as u8;

    pub fn all_free() -> Self {
        Self { alloc_masks: [0; Self::MASKS] }
    }

    pub fn alloc(&mut self) -> Result<Id, AllocError> {
        let mut id = Err(AllocError::Exhausted);
        for (block, mask) in self.alloc_masks.iter_mut().enumerate() {
            let trailing_ones = mask.trailing_ones() as u8;
            let is_last = block >= Self::MASKS - 1;
            let maximum = Self::MASK_BITS - u8::from(is_last);
            if trailing_ones < maximum {
                *mask |= 1 << trailing_ones;
                let index = (block as u8) * Self::MASK_BITS + trailing_ones;
                id = Ok(Id::new_raw(index + 1));
                break;
            }
        }
        id
    }

    fn block(id: Id) -> usize {
        let index = id.bits() - 1;
        usize::from(index) / Self::MASKS
    }

    fn bit(id: Id) -> RegistryMask {
        let index = id.bits() - 1;
        RegistryMask::from(index % Self::MASK_BITS)
    }

    pub fn is_allocated(&self, id: Id) -> bool {
        self.alloc_masks[Self::block(id)] & Self::bit(id) != 0
    }

    pub fn free(&mut self, id: Id) -> Result<(), DeallocError> {
        if self.is_allocated(id) {
            Err(DeallocError::AlreadyFree(id))?
        }
        self.alloc_masks[Self::block(id)] &= !Self::bit(id);
        Ok(())
    }
}
