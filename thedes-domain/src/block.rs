#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Block {
    Player = 0,
}

impl Block {
    pub const ALL: [Self; 1] = [Self::Player];
}
