use std::{collections::HashMap, fmt};

use smallstr::SmallString;
use thiserror::Error;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

type Grapheme = SmallString<[u8; 8]>;

#[derive(Debug, Error)]
#[error("Input {input} is not a grapheme")]
pub struct NotGrapheme {
    pub input: String,
}

#[derive(Debug, Error)]
#[error("Grapheme id {id} is unknwon")]
pub struct UnknownId {
    pub id: Id,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id {
    index: usize,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}

#[derive(Debug)]
pub struct Registry {
    index_to_string: Vec<Grapheme>,
    string_to_index: HashMap<Grapheme, Id>,
}

impl Registry {
    pub(crate) fn new() -> Self {
        Self { index_to_string: Vec::new(), string_to_index: HashMap::new() }
    }

    pub fn get_or_register_many<'r, 'g>(
        &'r mut self,
        graphemes: &'g str,
    ) -> GetOrRegisterMany<'r, 'g> {
        GetOrRegisterMany {
            registry: self,
            graphemes: graphemes.graphemes(true),
        }
    }

    pub fn get_or_register(
        &mut self,
        grapheme: &str,
    ) -> Result<Id, NotGrapheme> {
        let mut iter = grapheme.graphemes(true);
        if iter.next().is_none() {
            Err(NotGrapheme { input: grapheme.into() })?;
        }
        if iter.next().is_some() {
            Err(NotGrapheme { input: grapheme.into() })?;
        }
        Ok(self.get_or_register_unchecked(grapheme))
    }

    pub fn lookup(&self, id: Id) -> Result<&str, UnknownId> {
        self.index_to_string
            .get(id.index)
            .map(AsRef::as_ref)
            .ok_or(UnknownId { id })
    }

    fn get_or_register_unchecked(&mut self, grapheme: &str) -> Id {
        match self.string_to_index.get(grapheme) {
            Some(id) => *id,
            None => self.register_unchecked(grapheme),
        }
    }

    fn register_unchecked(&mut self, grapheme: &str) -> Id {
        let index = self.index_to_string.len();
        let id = Id { index };
        self.index_to_string.push(grapheme.into());
        self.string_to_index.insert(grapheme.into(), id);
        id
    }
}

#[derive(Debug)]
pub struct GetOrRegisterMany<'r, 'g> {
    registry: &'r mut Registry,
    graphemes: Graphemes<'g>,
}

impl<'r, 'g> Iterator for GetOrRegisterMany<'r, 'g> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let grapheme = self.graphemes.next()?;
        Some(self.registry.get_or_register_unchecked(grapheme))
    }
}
