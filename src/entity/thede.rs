use super::Id;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
/// A thede. A thede is tribe, a people or a nation.
pub struct Thede {
    id: Id,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
/// A village, belonging to a thede.
pub struct Village {
    thede: Id,
    id: Id,
}
