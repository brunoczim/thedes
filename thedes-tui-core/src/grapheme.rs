use std::{collections::HashMap, fmt, str, sync::Arc};

use thiserror::Error;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

type Grapheme = Box<str>;

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
    bits: u64,
}

impl Id {
    fn from_index(index: usize) -> Self {
        let max_code = u64::from(char::MAX);
        let bits = u64::try_from(index)
            .ok()
            .and_then(|bits| bits.checked_add(max_code + 1))
            .expect("index could not be so large");
        Self { bits }
    }
}

impl From<char> for Id {
    fn from(value: char) -> Self {
        Self { bits: u64::from(value) }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bits)
    }
}

impl PartialEq<char> for Id {
    fn eq(&self, other: &char) -> bool {
        *self == Self::from(*other)
    }
}

impl PartialEq<Id> for char {
    fn eq(&self, other: &Id) -> bool {
        Id::from(*self) == *other
    }
}

#[derive(Debug)]
struct RegistryInner {
    index_to_string: Vec<Grapheme>,
    string_to_id: HashMap<Grapheme, Id>,
}

impl RegistryInner {
    pub fn new() -> Self {
        Self { index_to_string: Vec::new(), string_to_id: HashMap::new() }
    }

    pub fn index_to_string(&self, index: usize) -> Option<&str> {
        self.index_to_string.get(index).map(AsRef::as_ref)
    }

    pub fn get_or_register(&mut self, grapheme: &str) -> Id {
        match self.string_to_id.get(grapheme) {
            Some(id) => *id,
            None => self.register(grapheme),
        }
    }

    pub fn register(&mut self, grapheme: &str) -> Id {
        let index = self.index_to_string.len();
        let id = Id::from_index(index);
        self.index_to_string.push(grapheme.into());
        self.string_to_id.insert(grapheme.into(), id);
        id
    }
}

#[derive(Debug, Clone)]
pub struct Registry {
    inner: Arc<std::sync::Mutex<RegistryInner>>,
}

impl Registry {
    pub fn new() -> Self {
        Self { inner: Arc::new(std::sync::Mutex::new(RegistryInner::new())) }
    }

    pub fn get_or_register_many<'r, 'g>(
        &'r self,
        graphemes: &'g str,
    ) -> GetOrRegisterMany<'r, 'g> {
        GetOrRegisterMany {
            registry: self,
            graphemes: graphemes.graphemes(true),
        }
    }

    pub fn get_or_register(&self, grapheme: &str) -> Result<Id, NotGrapheme> {
        let mut iter = grapheme.graphemes(true);
        if iter.next().is_none() {
            Err(NotGrapheme { input: grapheme.into() })?;
        }
        if iter.next().is_some() {
            Err(NotGrapheme { input: grapheme.into() })?;
        }
        Ok(self.get_or_register_unchecked(grapheme))
    }

    pub fn lookup<F, T>(&self, id: Id, scope: F) -> T
    where
        F: FnOnce(Result<GraphemeChars<'_>, UnknownId>) -> T,
    {
        let max_char = u64::from(char::MAX);
        if let Some(bits) = id.bits.checked_sub(max_char + 1) {
            let index = usize::try_from(bits)
                .expect("id bits should have been constructed from index");
            let inner = self.inner.lock().expect("poisoned lock");
            let result = inner
                .index_to_string(index)
                .map(GraphemeChars::multiple)
                .ok_or(UnknownId { id });
            scope(result)
        } else {
            let code = u32::try_from(id.bits)
                .ok()
                .and_then(char::from_u32)
                .expect("already checked for char range");
            let result = Ok(GraphemeChars::single(code));
            scope(result)
        }
    }

    fn get_or_register_unchecked(&self, grapheme: &str) -> Id {
        let mut chars = grapheme.chars();
        if let Some(ch) = chars.next() {
            if chars.next().is_none() {
                return Id::from(ch);
            }
        }

        let mut inner = self.inner.lock().expect("poisoned lock");
        inner.get_or_register(grapheme)
    }
}

#[derive(Debug)]
pub struct GetOrRegisterMany<'r, 'g> {
    registry: &'r Registry,
    graphemes: Graphemes<'g>,
}

impl<'r, 'g> Iterator for GetOrRegisterMany<'r, 'g> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let grapheme = self.graphemes.next()?;
        Some(self.registry.get_or_register_unchecked(grapheme))
    }
}

#[derive(Debug, Clone)]
pub struct GraphemeChars<'r> {
    inner: GraphemeCharsInner<'r>,
}

impl<'r> GraphemeChars<'r> {
    fn single(ch: char) -> Self {
        Self { inner: GraphemeCharsInner::Single(Some(ch)) }
    }

    fn multiple(content: &'r str) -> Self {
        Self { inner: GraphemeCharsInner::Multiple(content.chars()) }
    }
}

impl<'r> Iterator for GraphemeChars<'r> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            GraphemeCharsInner::Single(ch) => ch.take(),
            GraphemeCharsInner::Multiple(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            GraphemeCharsInner::Single(None) => (0, Some(0)),
            GraphemeCharsInner::Single(Some(_)) => (1, Some(1)),
            GraphemeCharsInner::Multiple(iter) => iter.size_hint(),
        }
    }
}

#[derive(Debug, Clone)]
enum GraphemeCharsInner<'r> {
    Single(Option<char>),
    Multiple(str::Chars<'r>),
}

#[cfg(test)]
mod test {
    use super::{Id, Registry};

    #[test]
    fn id_from_char_is_char_ascii() {
        let actual = Id::from('a').bits;
        let expected = 'a' as u64;
        assert_eq!(expected, actual);
    }

    #[test]
    fn id_from_char_is_char_unicode() {
        let actual = Id::from('á').bits;
        let expected = 'á' as u64;
        assert_eq!(expected, actual);
    }

    #[test]
    fn register_single_char_grapheme_ascii() {
        let registry = Registry::new();
        let id = registry.get_or_register("a").unwrap();
        let actual: String =
            registry.lookup(id, |result| result.unwrap().collect());
        let expected = "a";
        assert_eq!(expected, actual);

        let expected = "a";
        let actual: String =
            registry.lookup(id, |result| result.unwrap().collect());
        assert_eq!(expected, actual);
    }

    #[test]
    fn register_single_char_grapheme_unicode() {
        let registry = Registry::new();
        let id = registry.get_or_register("á").unwrap();
        let actual: String =
            registry.lookup(id, |result| result.unwrap().collect());
        let expected = "á";
        assert_eq!(expected, actual);

        let expected = "á";
        let actual: String =
            registry.lookup(id, |result| result.unwrap().collect());
        assert_eq!(expected, actual);
    }

    #[test]
    fn register_single_grapheme_cluster() {
        let registry = Registry::new();
        let id = registry.get_or_register("b̥").unwrap();
        let actual: String =
            registry.lookup(id, |result| result.unwrap().collect());
        let expected = "b̥";
        assert_eq!(expected, actual);

        let expected = "b̥";
        let actual: String =
            registry.lookup(id, |result| result.unwrap().collect());
        assert_eq!(expected, actual);
    }

    #[test]
    fn register_many() {
        let registry = Registry::new();
        let ids: Vec<_> = registry.get_or_register_many("ab̥á").collect();
        let mut actual = Vec::<String>::new();
        for &id in &ids {
            let actual_elem =
                registry.lookup(id, |result| result.unwrap().collect());
            actual.push(actual_elem);
        }
        let expected = ["a", "b̥", "á"].map(ToOwned::to_owned);
        assert_eq!(&expected[..], &actual[..]);

        let expected = "a";
        let actual: String =
            registry.lookup(ids[0], |result| result.unwrap().collect());
        assert_eq!(expected, actual);

        let expected = "b̥";
        let actual: String =
            registry.lookup(ids[1], |result| result.unwrap().collect());
        assert_eq!(expected, actual);

        let expected = "á";
        let actual: String =
            registry.lookup(ids[2], |result| result.unwrap().collect());
        assert_eq!(expected, actual);
    }
}
