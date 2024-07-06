use num_derive::{FromPrimitive, ToPrimitive};

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
    ToPrimitive,
)]
#[repr(u8)]
pub enum Ground {
    Grass = 0,
    Sand = 1,
    Stone = 2,
}

impl Default for Ground {
    fn default() -> Self {
        Self::Grass
    }
}
