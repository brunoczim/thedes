use super::language::Language;
use std::fmt;

/// ID of a thede.
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
pub struct Id(u16);

impl fmt::Display for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, fmt)
    }
}

/// A thede's data.
#[derive(Debug, Clone)]
pub struct Thede {
    id: Id,
    data: ThedeData,
}

impl Thede {
    pub fn new(id: Id, data: ThedeData) -> Self {
        Self { id, data }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThedeData {
    hash: u64,
    language: Language,
}
