use std::{
    cmp,
    collections::{BinaryHeap, HashMap},
    fmt,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
#[error("Identifier {0} is not short")]
pub struct NonShortId(pub Id);

#[derive(Debug, Clone, Copy, Error)]
#[error("Identifier {0} is not tiny")]
pub struct NonTinyId(pub Id);

#[derive(Debug, Clone, Copy, Error)]
#[error("Invalid identifier {0}")]
pub struct InvalidId(pub Id);

#[derive(Debug, Clone, Copy, Error)]
#[error("Invalid index {index} out of length {len}")]
pub struct InvalidIndex {
    pub index: usize,
    pub len: usize,
}

#[derive(Debug, Clone, Copy, Error)]
pub enum InvalidIndexAs<E> {
    #[error(transparent)]
    OutOfBounds(#[from] InvalidIndex),
    #[error("id is not convertible")]
    IdConversion(#[source] E),
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
pub struct Id(u32);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ShortId> for Id {
    fn from(id: ShortId) -> Self {
        Self(id.0.into())
    }
}

impl From<TinyId> for Id {
    fn from(id: TinyId) -> Self {
        Self(id.0.into())
    }
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
pub struct ShortId(u16);

impl fmt::Display for ShortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TinyId> for ShortId {
    fn from(id: TinyId) -> Self {
        Self(id.0.into())
    }
}

impl TryFrom<Id> for ShortId {
    type Error = NonShortId;

    fn try_from(id: Id) -> Result<Self, Self::Error> {
        match id.0.try_into() {
            Ok(bits) => Ok(Self(bits)),
            Err(_) => Err(NonShortId(id)),
        }
    }
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
pub struct TinyId(u8);

impl TryFrom<Id> for TinyId {
    type Error = NonTinyId;

    fn try_from(id: Id) -> Result<Self, Self::Error> {
        match id.0.try_into() {
            Ok(bits) => Ok(Self(bits)),
            Err(_) => Err(NonTinyId(id)),
        }
    }
}

impl TryFrom<ShortId> for TinyId {
    type Error = NonTinyId;

    fn try_from(id: ShortId) -> Result<Self, Self::Error> {
        match id.0.try_into() {
            Ok(bits) => Ok(Self(bits)),
            Err(_) => Err(NonTinyId(id.into())),
        }
    }
}

impl fmt::Display for TinyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry<T> {
    primary: Vec<(Id, T)>,
    secondary_pos: HashMap<Id, usize>,
    secondary_neg: BinaryHeap<cmp::Reverse<Id>>,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            primary: Vec::new(),
            secondary_pos: HashMap::new(),
            secondary_neg: BinaryHeap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.primary.len()
    }

    pub fn create(&mut self, data: T) -> Id {
        let id = self.pop_secondary_neg();
        self.create_with_id(data, id);
        id
    }

    pub fn create_as<I, E>(&mut self, data: T) -> Result<I, E>
    where
        Id: TryInto<I, Error = E>,
    {
        let id = self.pop_secondary_neg();
        match id.try_into() {
            Ok(converted_id) => {
                self.create_with_id(data, id);
                Ok(converted_id)
            },
            Err(error) => {
                self.push_secondary_neg(id);
                Err(error)
            },
        }
    }

    pub fn remove(&mut self, id: impl Into<Id>) -> Result<T, InvalidId> {
        let id = id.into();
        let index =
            self.secondary_pos.get(&id).copied().ok_or(InvalidId(id))?;
        let last_index = self.primary.len() - 1;
        if index != last_index {
            self.primary.swap(index, last_index);
            let (id, _) = self.primary[index];
            self.secondary_pos.insert(id, index);
        }
        let (_, item) = self.primary.pop().expect("inconsistent tables");
        self.push_secondary_neg(id);
        Ok(item)
    }

    pub fn id_to_index(&self, id: impl Into<Id>) -> Result<usize, InvalidId> {
        let id = id.into();
        self.secondary_pos.get(&id).copied().ok_or(InvalidId(id))
    }

    pub fn get_by_id(&self, id: impl Into<Id>) -> Result<&T, InvalidId> {
        let index = self.id_to_index(id)?;
        Ok(&self.primary[index].1)
    }

    pub fn get_by_id_mut(
        &mut self,
        id: impl Into<Id>,
    ) -> Result<&mut T, InvalidId> {
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

    pub fn get_by_index_as<I, E>(
        &self,
        index: usize,
    ) -> Result<(I, &T), InvalidIndexAs<E>>
    where
        Id: TryInto<I, Error = E>,
    {
        let (id, data) = self.get_by_index(index)?;
        let converted_id =
            id.try_into().map_err(InvalidIndexAs::IdConversion)?;
        Ok((converted_id, data))
    }

    pub fn get_by_index_mut_as<I, E>(
        &mut self,
        index: usize,
    ) -> Result<(I, &mut T), InvalidIndexAs<E>>
    where
        Id: TryInto<I, Error = E>,
    {
        let (id, data) = self.get_by_index_mut(index)?;
        let converted_id =
            id.try_into().map_err(InvalidIndexAs::IdConversion)?;
        Ok((converted_id, data))
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

    fn create_with_id(&mut self, data: T, id: Id) {
        let index = self.primary.len();
        self.primary.push((id, data));
        self.secondary_pos.insert(id, index);
    }

    fn pop_secondary_neg(&mut self) -> Id {
        match self.secondary_neg.pop() {
            Some(cmp::Reverse(id)) => id,
            None => Id(self.len().try_into().expect("too much ids")),
        }
    }

    fn push_secondary_neg(&mut self, id: Id) {
        self.secondary_neg.push(cmp::Reverse(id));
    }
}
