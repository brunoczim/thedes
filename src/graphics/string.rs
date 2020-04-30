use crate::{
    error::{Error, Result},
    graphics::{Tile, UpdateColors},
};
use lazy_static::lazy_static;
use std::{
    cmp::Ordering,
    convert::TryFrom,
    error::Error as StdError,
    fmt,
    hash::{Hash, Hasher},
    iter::FromIterator,
    ops::{Deref, Range, RangeFrom, RangeFull, RangeTo},
    path::Path,
    slice,
    sync::Arc,
};
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

#[inline(never)]
#[cold]
fn index_panic<I>(count: usize, index: I) -> !
where
    I: fmt::Debug,
{
    panic!(
        "GString index out of bounds: grapheme count is {} but index {:?}",
        count, index
    )
}

/// Graphical string: a string valid to be printed on a terminal for graphic
/// purpouse.
#[derive(Debug, Clone)]
pub struct GString {
    alloc: Arc<str>,
    range: Range<usize>,
}

impl GString {
    /// Builds a new graphical string.
    ///
    /// The string must not start with a diacritic character. Diacritic here is
    /// not "^" or "~", but rather a diacritic that when inserted combines with
    /// the previous character. Like the tilde in "ỹ" which can be separated
    /// from "y". On the other hand, the combination "ỹ" is valid and forms a
    /// single grapheme. The diacritic is only invalid when separated.
    ///
    /// Control characters also trigger an error, because those would allow the
    /// terminal to be controlled.
    pub fn new<S>(string: S) -> Result<Self>
    where
        S: Into<String> + AsRef<str>,
    {
        for ch in string.as_ref().chars() {
            if ch.is_control() {
                Err(InvalidControl)?;
            }
        }

        let mut new_string = string.into();
        new_string.replace_range(0 .. 0, "a");
        let mut iter = new_string.grapheme_indices(true);
        iter.next();
        let index = iter.next().map_or(new_string.len(), |(index, _)| index);
        if index != 1 {
            Err(StartsWithDiacritic)?;
        }
        new_string.replace_range(0 .. 1, "");

        let range = 0 .. new_string.len();
        Ok(GString { alloc: new_string.into(), range })
    }

    /// Creates a new `GString`, but replaces error with the replacement
    /// character "�".
    pub fn new_lossy<S>(string: S) -> Self
    where
        S: AsRef<str>,
    {
        let mut new_string = String::new();
        for ch in string.as_ref().chars() {
            new_string.push(if ch.is_control() { '�' } else { ch });
        }

        new_string.replace_range(0 .. 0, "a");
        let mut iter = new_string.grapheme_indices(true);
        iter.next();
        let index = iter.next().map_or(new_string.len(), |(index, _)| index);
        let replacement = if index != 1 { "�" } else { "" };
        new_string.replace_range(0 .. 1, replacement);

        let range = 0 .. new_string.len();
        GString { alloc: new_string.into(), range }
    }

    /// Counts how many graphemes the string contains by iterating the string.
    pub fn count_graphemes(&self) -> usize {
        self.as_str().graphemes(true).count()
    }

    /// Converts into a reference to a plain string.
    pub fn as_str(&self) -> &str {
        &self.alloc[self.range.clone()]
    }

    /// Indexes the string by returning `None` if out of bounds. `usize` will
    /// return `Grapheme`s, ranges will return sub-`GString`s. WARNING: this
    /// method is, prefere iterating instead.
    pub fn get<I>(&self, index: I) -> Option<I::Output>
    where
        I: Index,
    {
        index.get(self)
    }

    /// Indexes the string by panicking if out of bounds. `usize` will
    /// return `Grapheme`s, ranges will return sub-`GString`s. WARNING: this
    /// method is slow, prefere iterating instead.
    pub fn index<I>(&self, index: I) -> I::Output
    where
        I: Index,
    {
        index.index(self)
    }

    /// Iterates over indices and graphemes of this string.
    pub fn indices(&self) -> GStringIndices {
        let mut indices = self.as_str().grapheme_indices(true);
        let prev_index = indices.next().map_or(self.len(), |(index, _)| index);
        let next_index = self.len();

        GStringIndices { indices, prev_index, next_index, base: self.clone() }
    }

    /// Iterates only over graphemes of this string.
    pub fn iter(&self) -> GStringIter {
        self.into_iter()
    }

    /// De-slices a sub-`GString` into the original buffer.
    pub fn full_buf(self) -> Self {
        Self { alloc: self.alloc.clone(), range: 0 .. self.alloc.len() }
    }
}

impl FromIterator<Grapheme> for GString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Grapheme>,
    {
        let mut buf = String::new();
        for grapheme in iter {
            buf.push_str(grapheme.as_str());
        }
        let range = 0 .. buf.len();
        Self { alloc: buf.into(), range }
    }
}

impl FromIterator<GString> for GString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = GString>,
    {
        let mut buf = String::new();
        for gstr in iter {
            buf.push_str(gstr.as_str());
        }
        let range = 0 .. buf.len();
        Self { alloc: buf.into(), range }
    }
}

impl<'buf> FromIterator<&'buf Grapheme> for GString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'buf Grapheme>,
    {
        let mut buf = String::new();
        for grapheme in iter {
            buf.push_str(grapheme.as_str());
        }
        let range = 0 .. buf.len();
        Self { alloc: buf.into(), range }
    }
}

impl<'buf> FromIterator<&'buf GString> for GString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'buf GString>,
    {
        let mut buf = String::new();
        for gstr in iter {
            buf.push_str(gstr.as_str());
        }
        let range = 0 .. buf.len();
        Self { alloc: buf.into(), range }
    }
}

impl<'buf> FromIterator<StringOrGraphm<'buf>> for GString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = StringOrGraphm<'buf>>,
    {
        // also a heuristics
        let mut buf = String::with_capacity(80);
        for gstr in iter {
            buf.push_str(gstr.as_str());
        }
        let range = 0 .. buf.len();
        Self { alloc: buf.into(), range }
    }
}

impl Default for GString {
    fn default() -> Self {
        Self { alloc: Arc::from(""), range: 0 .. 0 }
    }
}

impl AsRef<str> for GString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for GString {
    fn as_ref(&self) -> &Path {
        self.as_str().as_ref()
    }
}

impl Deref for GString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl PartialEq for GString {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.alloc, &other.alloc) && self.range == other.range
            || self.as_str() == other.as_str()
    }
}

impl Eq for GString {}

impl PartialOrd for GString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for GString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for GString {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_str().hash(state)
    }
}

impl fmt::Display for GString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl<'buf> TryFrom<&'buf str> for GString {
    type Error = Error;

    fn try_from(buf: &'buf str) -> Result<Self> {
        Self::new(buf)
    }
}

impl TryFrom<String> for GString {
    type Error = Error;

    fn try_from(buf: String) -> Result<Self> {
        Self::new(buf)
    }
}

/// A grapheme cluster. Represents what a human visually sees as a character.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Grapheme {
    gstring: GString,
}

lazy_static! {
    static ref DEFAULT_GRAPHEME: Grapheme =
        Grapheme { gstring: GString { alloc: Arc::from(" "), range: 0 .. 1 } };
}

impl Default for Grapheme {
    /// Returns the grapheme for the space " ".
    fn default() -> Self {
        DEFAULT_GRAPHEME.clone()
    }
}

impl fmt::Display for Grapheme {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.gstring)
    }
}

impl Deref for Grapheme {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Grapheme {
    /// Builds a new grapheme cluster. The argument must be composed of only one
    /// grapheme.
    ///
    /// The string must not start with a diacritic character.
    /// Diacritic here is not "^" or "~", but rather a diacritic that when
    /// inserted combines with the previous character. Like the tilde in "ỹ"
    /// which can be separated from "y". On the other hand, the combination "ỹ"
    /// is valid and forms a single grapheme. The diacritic is only invalid when
    /// separated.
    ///
    /// Control characters also trigger an error, because those would allow the
    /// terminal to be controlled.
    pub fn new<S>(string: S) -> Result<Self>
    where
        S: AsRef<str> + Into<String>,
    {
        let first = string
            .as_ref()
            .graphemes(true)
            .next()
            .ok_or_else(|| NotAGrapheme)?;
        if first.len() != string.as_ref().len() {
            Err(NotAGrapheme)?;
        }

        for ch in string.as_ref().chars() {
            if ch.is_control() {
                Err(InvalidControl)?;
            }
        }

        let mut new_string = string.into();
        new_string.replace_range(0 .. 0, "a");
        let count = new_string.graphemes(true).count();
        if count == 1 {
            Err(StartsWithDiacritic)?;
        }
        new_string.replace_range(0 .. 1, "");

        let range = 0 .. new_string.len();
        let gstring = GString { alloc: new_string.into(), range };
        Ok(Self { gstring })
    }

    /// Creates a new `Grapheme`, but replaces error with the replacement
    /// character "�". Truncates the string it contains more than one grapheme.
    pub fn new_lossy<S>(string: S) -> Self
    where
        S: Into<String> + AsRef<str>,
    {
        let actual_str = string.as_ref().graphemes(true).next().unwrap_or("�");
        let mut new_string = String::new();
        for ch in actual_str.chars() {
            new_string.push(if ch.is_control() { '�' } else { ch });
        }

        new_string.replace_range(0 .. 0, "a");
        let mut iter = new_string.grapheme_indices(true);
        iter.next();
        let index = iter.next().map_or(new_string.len(), |(index, _)| index);
        let replacement = if index != 1 { "�" } else { "" };
        new_string.replace_range(0 .. 1, replacement);

        let range = 0 .. new_string.len();
        let gstring = GString { alloc: new_string.into(), range };
        Self { gstring }
    }

    /// Returns the grapheme for the space " ". This is the default grapheme,
    /// used in `Default`.
    pub fn space() -> Grapheme {
        Self::default()
    }

    /// Converts into a reference of a plain string.
    pub fn as_str(&self) -> &str {
        &self.gstring
    }

    /// Returns the underlying string buffer of this `Grapheme`.
    pub fn as_gstring(&self) -> &GString {
        &self.gstring
    }

    /// Converts into the underlying string buffer of this `Grapheme`.
    pub fn into_gstring(self) -> GString {
        self.gstring
    }
}

impl<'buf> TryFrom<&'buf str> for Grapheme {
    type Error = Error;

    fn try_from(buf: &'buf str) -> Result<Self> {
        Self::new(buf)
    }
}

impl TryFrom<String> for Grapheme {
    type Error = Error;

    fn try_from(buf: String) -> Result<Self> {
        Self::new(buf)
    }
}

/// Error generated when validating a `GString` or a grapheme and the string
/// starts with a diacrtic.
#[derive(Debug, Clone, Default)]
pub struct StartsWithDiacritic;

impl StdError for StartsWithDiacritic {}

impl fmt::Display for StartsWithDiacritic {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "The given string with a diacritic")
    }
}

/// Error generated when validating a grapheme and the string does not
/// containing exactly one grapheme cluster.
#[derive(Debug, Clone, Default)]
pub struct NotAGrapheme;

impl StdError for NotAGrapheme {}

impl fmt::Display for NotAGrapheme {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "The given string is not made of only one grapheme cluster")
    }
}

/// Error generated when validating a `GString` and the string contains a
/// control byte.
#[derive(Debug, Clone, Default)]
pub struct InvalidControl;

impl StdError for InvalidControl {}

impl fmt::Display for InvalidControl {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "The string contains control characters")
    }
}

pub trait Index {
    type Output;

    fn get(self, gstring: &GString) -> Option<Self::Output>;
    fn index(self, gstring: &GString) -> Self::Output;
}

impl Index for usize {
    type Output = Grapheme;

    fn get(self, gstring: &GString) -> Option<Self::Output> {
        gstring.into_iter().nth(self)
    }

    fn index(self, gstring: &GString) -> Self::Output {
        self.get(gstring)
            .unwrap_or_else(|| index_panic(gstring.count_graphemes(), self))
    }
}

impl Index for Range<usize> {
    type Output = GString;

    fn get(self, gstring: &GString) -> Option<Self::Output> {
        let mut iter = gstring.indices();
        for _ in 0 .. self.start {
            iter.next()?;
        }
        let (start, _) = iter.next()?;
        for _ in self.start + 1 .. self.end {
            iter.next()?;
        }
        let end = iter.next().map_or(gstring.len(), |(index, _)| index);
        let range = start + gstring.range.start .. end + gstring.range.start;
        Some(GString { alloc: gstring.alloc.clone(), range })
    }

    fn index(self, gstring: &GString) -> Self::Output {
        self.clone()
            .get(gstring)
            .unwrap_or_else(|| index_panic(gstring.count_graphemes(), self))
    }
}

impl Index for RangeTo<usize> {
    type Output = GString;

    fn get(self, gstring: &GString) -> Option<Self::Output> {
        (0 .. self.end).get(gstring)
    }

    fn index(self, gstring: &GString) -> Self::Output {
        self.clone()
            .get(gstring)
            .unwrap_or_else(|| index_panic(gstring.count_graphemes(), self))
    }
}

impl Index for RangeFrom<usize> {
    type Output = GString;

    fn get(self, gstring: &GString) -> Option<Self::Output> {
        let mut iter = gstring.indices();
        for _ in 0 .. self.start {
            iter.next()?;
        }
        let start = iter.next().map_or(gstring.alloc.len(), |(index, _)| index);
        let end = gstring.alloc.len();
        let range = start + gstring.range.start .. end + gstring.range.start;
        Some(GString { alloc: gstring.alloc.clone(), range })
    }

    fn index(self, gstring: &GString) -> Self::Output {
        self.clone()
            .get(gstring)
            .unwrap_or_else(|| index_panic(gstring.count_graphemes(), self))
    }
}

impl Index for RangeFull {
    type Output = GString;

    fn get(self, gstring: &GString) -> Option<Self::Output> {
        Some(gstring.clone())
    }

    fn index(self, gstring: &GString) -> Self::Output {
        gstring.clone()
    }
}

/// Iterator over the `GString`'s grapheme clusters indices and over the
/// grapheme clusters themselves.
pub struct GStringIndices<'gstring> {
    base: GString,
    prev_index: usize,
    next_index: usize,
    indices: GraphemeIndices<'gstring>,
}

impl<'gstring> Iterator for GStringIndices<'gstring> {
    type Item = (usize, Grapheme);

    fn next(&mut self) -> Option<Self::Item> {
        if self.prev_index == self.next_index {
            None
        } else {
            let index =
                self.indices.next().map_or(self.next_index, |(index, _)| index);
            let start = self.base.range.start + self.prev_index;
            let end = self.base.range.start + index;
            let alloc = self.base.alloc.clone();
            let gstring = GString { alloc, range: start .. end };
            self.prev_index = index;
            Some((gstring.range.start, Grapheme { gstring }))
        }
    }
}

impl<'gstring> DoubleEndedIterator for GStringIndices<'gstring> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.prev_index == self.next_index {
            None
        } else {
            let index = self
                .indices
                .next_back()
                .map_or(self.prev_index, |(index, _)| index);
            let start = self.base.range.start + index;
            let end = self.base.range.start + self.next_index;
            let alloc = self.base.alloc.clone();
            let gstring = GString { alloc, range: start .. end };
            self.next_index = index;
            Some((gstring.range.start, Grapheme { gstring }))
        }
    }
}

impl<'gstring> fmt::Debug for GStringIndices<'gstring> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("GStringIndices")
            .field("base", &self.base)
            .field("prev_index", &self.prev_index)
            .field("next_index", &self.next_index)
            .finish()
    }
}

/// Iterator only over the grapheme clusters of a `GString`.
#[derive(Debug)]
pub struct GStringIter<'gstring> {
    inner: GStringIndices<'gstring>,
}

impl<'gstring> Iterator for GStringIter<'gstring> {
    type Item = Grapheme;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, grapheme)| grapheme)
    }
}

impl<'gstring> DoubleEndedIterator for GStringIter<'gstring> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(_, grapheme)| grapheme)
    }
}

impl<'gstring> IntoIterator for &'gstring GString {
    type Item = Grapheme;
    type IntoIter = GStringIter<'gstring>;

    fn into_iter(self) -> Self::IntoIter {
        GStringIter { inner: self.indices() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringOrGraphm<'buf> {
    Graphm(&'buf Grapheme),
    String(&'buf GString),
}

impl<'buf> StringOrGraphm<'buf> {
    pub fn as_str(self) -> &'buf str {
        match self {
            StringOrGraphm::Graphm(grapheme) => grapheme.as_str(),
            StringOrGraphm::String(gstr) => gstr.as_str(),
        }
    }

    pub fn as_gstring(self) -> &'buf GString {
        match self {
            StringOrGraphm::Graphm(grapheme) => grapheme.as_gstring(),
            StringOrGraphm::String(gstr) => gstr,
        }
    }
}

impl<'buf> AsRef<str> for StringOrGraphm<'buf> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'buf> AsRef<GString> for StringOrGraphm<'buf> {
    fn as_ref(&self) -> &GString {
        self.as_gstring()
    }
}

impl<'buf> From<&'buf Grapheme> for StringOrGraphm<'buf> {
    fn from(grapheme: &'buf Grapheme) -> StringOrGraphm<'buf> {
        StringOrGraphm::Graphm(grapheme)
    }
}

impl<'buf> From<&'buf GString> for StringOrGraphm<'buf> {
    fn from(gstring: &'buf GString) -> StringOrGraphm<'buf> {
        StringOrGraphm::String(gstring)
    }
}

/// A String that can have different colors in different slices of it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColoredGString<C>
where
    C: UpdateColors,
{
    gstring: GString,
    colors: Vec<(usize, C)>,
}

impl<C> ColoredGString<C>
where
    C: UpdateColors,
{
    /// Creates a new string from a list of pairs of `GString`s and color
    /// updates.
    pub fn new<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = (GString, C)>,
    {
        let iter = iterable.into_iter();
        let (min, _) = iter.size_hint();
        let mut buf = String::with_capacity(min * 8);
        let mut colors = Vec::with_capacity(min);

        for (string, color) in iter {
            colors.push((string.len(), color));
            buf += string.as_str();
        }

        Self {
            gstring: GString { range: 0 .. buf.len(), alloc: buf.into() },
            colors,
        }
    }

    /// Create a colored `GString` from an existing [`GString`] and a sequence
    /// of pairs of a segment's size (in graphemes) and its color.
    ///
    /// # Panics
    ///
    /// Panics if the total sum of grapheme counts does not match the number of
    /// graphemes in the string.
    fn from_gstring<I>(gstring: GString, color_sizes: I) -> Self
    where
        I: IntoIterator<Item = (usize, C)>,
    {
        Self::try_from_gstring(gstring, color_sizes)
            .expect("counts didn't match")
    }

    /// Tries to create a colored `GString` from an existing [`GString`] and a
    /// sequence of pairs of a segment's size (in graphemes) and its color. The
    /// total sum of grapheme counts must be equal to the number of graphemes in
    /// the string, otherwise [`CountNotMatched`] is returned.
    pub fn try_from_gstring<I>(gstring: GString, color_sizes: I) -> Result<Self>
    where
        I: IntoIterator<Item = (usize, C)>,
    {
        let mut colors_iter = color_sizes.into_iter();
        let (min, _) = colors_iter.size_hint();

        let mut indices_iter = gstring.indices();
        let mut colors = Vec::with_capacity(min);
        let mut last_index = 0;

        for (count, color) in colors_iter {
            colors.push((last_index, color));
            for _ in 0 .. count {
                match indices_iter.next() {
                    Some((index, _)) => last_index = index,
                    None => Err(CountNotMatched {
                        gstring: gstring.count_graphemes(),
                    })?,
                }
            }
        }

        if last_index != gstring.len() {
            Err(CountNotMatched { gstring: gstring.count_graphemes() })?
        }

        Ok(Self { gstring, colors })
    }

    /// The underlying [`GString`] that composes the text of this
    /// [`ColoredGString`].
    pub fn gstring(&self) -> &GString {
        &self.gstring
    }

    pub fn indices(&self) -> ColoredGStringIndices<C> {
        let mut colors = self.colors.iter();
        ColoredGStringIndices {
            curr: colors.next().map(|(index, color)| (*index, color)),
            next: colors.next().map(|(index, color)| (*index, color)),
            colors,
            indices: self.gstring.indices(),
        }
    }
}

/// Iterator over grapheme-level [`Tile`]s and their index.
#[derive(Debug, Clone)]
pub struct ColoredGStringIndices<'gstring, C>
where
    C: UpdateColors,
{
    curr: Option<(usize, &'gstring C)>,
    next: Option<(usize, &'gstring C)>,
    colors: slice::Iter<'gstring, (usize, C)>,
    indices: GStringIndices<'gstring>,
}

impl<'gstring, C> Iterator for ColoredGStringIndices<'gstring, C>
where
    C: UpdateColors,
{
    type Item = (usize, Tile<&'gstring C>);

    fn next(&mut self) -> Option<Self::Item> {
        let (mut curr, mut curr_colors) = self.curr?;
        let len = self.indices.base.len();
        let (mut next, next_colors) = self
            .next
            .map_or((len, None), |(index, colors)| (index, Some(colors)));
        if self.indices.prev_index >= next {
            curr = next;
            curr_colors = next_colors?;
            self.next =
                self.colors.next().map(|(count, colors)| (*count, colors));
        }

        let (index, grapheme) = self.indices.next()?;

        Some((index, Tile { grapheme, colors: curr_colors }))
    }
}

/// Error generated when validating a `ColoredGString` from a `GString` and an
/// iterator of counts, and the iterator total count does not match the
/// `GString` count of graphemes.
#[derive(Debug, Clone, Default)]
pub struct CountNotMatched {
    /// Grapheme count of the gstring.
    pub gstring: usize,
}

impl StdError for CountNotMatched {}

impl fmt::Display for CountNotMatched {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "The given text with grapheme count {} does not match colors \
             total count",
            self.gstring
        )
    }
}
