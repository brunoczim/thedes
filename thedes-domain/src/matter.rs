#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ground {
    Grass,
    Sand,
    Stone,
}

impl Default for Ground {
    fn default() -> Self {
        Self::Grass
    }
}
