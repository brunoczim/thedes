use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
#[error("Invalid identifier {0}")]
pub struct InvalidId(pub Id);

#[derive(Debug, Clone, Copy, Error)]
#[error("Invalid index {index} out of length {len}")]
pub struct InvalidIndex {
    pub index: usize,
    pub len: usize,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct Id(u64);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry<T> {
    primary: Vec<(Id, T)>,
    secondary_pos: HashMap<Id, usize>,
    secondary_neg: Id,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            primary: Vec::new(),
            secondary_pos: HashMap::new(),
            secondary_neg: Id(0),
        }
    }

    pub fn len(&self) -> usize {
        self.primary.len()
    }

    pub fn create(&mut self, data: T) -> Id {
        let id = self.secondary_neg;
        self.secondary_neg.0 += 1;
        let index = self.primary.len();
        self.primary.push((id, data));
        self.secondary_pos.insert(id, index);
        id
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let index = self.secondary_pos.get(&id).copied()?;
        let new_index = self.primary.len() - 1;
        if index != new_index {
            self.primary.swap(index, new_index);
            let (id, _) = self.primary[index];
            self.secondary_pos.insert(id, index);
        }
        let (_, item) = self.primary.pop().expect("inconsistent tables");
        Some(item)
    }

    pub fn id_to_index(&self, id: Id) -> Result<usize, InvalidId> {
        self.secondary_pos.get(&id).copied().ok_or(InvalidId(id))
    }

    pub fn get_by_id(&self, id: Id) -> Result<&T, InvalidId> {
        let index = self.id_to_index(id)?;
        Ok(&self.primary[index].1)
    }

    pub fn get_by_id_mut(&mut self, id: Id) -> Result<&mut T, InvalidId> {
        let index = self.id_to_index(id)?;
        Ok(&mut self.primary[index].1)
    }

    pub fn get_by_index(&self, index: usize) -> Result<(Id, &T), InvalidIndex> {
        let error = InvalidIndex { index, len: self.primary.len() };
        let (id, value) = self.primary.get(index).ok_or(error)?;
        Ok((*id, value))
    }

    pub fn get_by_index_mut(
        &mut self,
        index: usize,
    ) -> Result<(Id, &mut T), InvalidIndex> {
        let error = InvalidIndex { index, len: self.primary.len() };
        let (id, value) = self.primary.get_mut(index).ok_or(error)?;
        Ok((*id, value))
    }

    pub fn iter<'a>(
        &'a self,
    ) -> impl DoubleEndedIterator<Item = (Id, &'a T)> + 'a + Send + Sync
    where
        T: Send + Sync,
    {
        self.primary.iter().map(|(id, elem)| (*id, elem))
    }

    pub fn iter_mut<'a>(
        &'a mut self,
    ) -> impl DoubleEndedIterator<Item = (Id, &'a mut T)> + 'a + Send + Sync
    where
        T: Send + Sync,
    {
        self.primary.iter_mut().map(|(id, elem)| (*id, elem))
    }

    pub fn into_iter<'a>(
        self,
    ) -> impl DoubleEndedIterator<Item = (Id, T)> + 'a + Send + Sync
    where
        T: Send + Sync + 'a,
    {
        self.primary.into_iter().map(|(id, elem)| (id, elem))
    }
}
