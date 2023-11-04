use std::collections::BTreeMap;

use super::{
    map,
    player::{self, Player},
};

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct GameSnapshot {
    pub map: map::Slice,
    pub players: BTreeMap<player::Name, Player>,
}
