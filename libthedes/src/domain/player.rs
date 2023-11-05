use std::{
    cmp::Ordering,
    fmt,
    str::{self, FromStr},
};
use thiserror::Error;

use super::human;

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
    Error,
)]
pub enum InvalidName {
    #[error(
        "Given player name is too short, minimum is {}, found {}",
        Name::MIN_LEN,
        .0,
    )]
    TooShort(usize),
    #[error(
        "Given player name is too long, maximum is {}, found {}",
        Name::MAX_LEN,
        .0
    )]
    TooLong(usize),
    #[error(
        "Invalid character given, only ASCII alphabetic, '-' and '_' are \
        allowed, found {}",
        match str::from_utf8(&[b'\'', *.0, b'\'']) {
            Ok(ch) => ch,
            Err(_) => "non-ASCII"
        }
    )]
    InvalidChar(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name {
    bits: u64,
}

impl Name {
    pub const MIN_LEN: usize = 1;

    pub const MAX_LEN: usize = 10;

    const CHAR_BITS: u32 = 6;

    const CHARS_BITS: u32 = Self::CHAR_BITS * Self::MAX_LEN as u32;

    const HYPHEN_OFFSET: u8 = 0;

    const DIGITS_OFFSET: u8 = Self::HYPHEN_OFFSET + 1;

    const UPPER_OFFSET: u8 = Self::DIGITS_OFFSET + 10;

    const UNDERSCORE_OFFSET: u8 = Self::UPPER_OFFSET + 26;

    const LOWER_OFFSET: u8 = Self::UNDERSCORE_OFFSET + 1;

    pub const MIN: Self =
        Self { bits: Self::pack_parts(Self::MIN_LEN as u64, 0) };

    pub const MAX: Self =
        Self { bits: Self::pack_parts(Self::MAX_LEN as u64, u64::MAX) };

    const fn pack_parts(len: u64, packed_chars: u64) -> u64 {
        len | packed_chars
    }

    const fn unpack_parts(packed: u64) -> (u64, u64) {
        let shift = u64::BITS - Self::CHARS_BITS;
        let mask = (1 << shift) - 1;
        let len = packed & mask;
        let packed_chars = packed & !mask;
        (len, packed_chars)
    }

    const fn pack_char(ascii_char: u8) -> Result<u8, InvalidName> {
        if ascii_char == b'-' {
            Ok(Self::HYPHEN_OFFSET)
        } else if ascii_char >= b'0' && ascii_char <= b'9' {
            Ok(ascii_char - b'0' + Self::DIGITS_OFFSET)
        } else if ascii_char >= b'A' && ascii_char <= b'Z' {
            Ok(ascii_char - b'A' + Self::UPPER_OFFSET)
        } else if ascii_char == b'_' {
            Ok(Self::UNDERSCORE_OFFSET)
        } else if ascii_char >= b'a' && ascii_char <= b'z' {
            Ok(ascii_char - b'a' + Self::LOWER_OFFSET)
        } else {
            Err(InvalidName::InvalidChar(ascii_char))
        }
    }

    const fn unpack_char(packed_char: u8) -> u8 {
        if packed_char >= Self::LOWER_OFFSET {
            packed_char - Self::LOWER_OFFSET + b'a'
        } else if packed_char == Self::UNDERSCORE_OFFSET {
            b'_'
        } else if packed_char >= Self::UPPER_OFFSET {
            packed_char - Self::UPPER_OFFSET + b'A'
        } else if packed_char >= Self::DIGITS_OFFSET {
            packed_char - Self::DIGITS_OFFSET + b'0'
        } else {
            b'-'
        }
    }

    pub const fn new(ascii_chars: &[u8]) -> Result<Self, InvalidName> {
        if ascii_chars.len() < Self::MIN_LEN {
            return Err(InvalidName::TooShort(ascii_chars.len()));
        }

        if ascii_chars.len() > Self::MAX_LEN {
            return Err(InvalidName::TooLong(ascii_chars.len()));
        }

        let mut packed_chars = 0;
        let mut i = ascii_chars.len();

        while i > 0 {
            i -= 1;
            packed_chars >>= Self::CHAR_BITS;
            packed_chars |= match Self::pack_char(ascii_chars[i]) {
                Ok(packed) => (packed as u64) << (u64::BITS - Self::CHAR_BITS),
                Err(error) => return Err(error),
            };
        }

        let bits = Self::pack_parts(ascii_chars.len() as u64, packed_chars);

        Ok(Self { bits })
    }

    pub const fn len(self) -> usize {
        let (len, _) = Self::unpack_parts(self.bits);
        len as usize
    }

    pub const fn ascii_chars(self) -> NameAsciiChars {
        let (len, packed_chars) = Self::unpack_parts(self.bits);
        NameAsciiChars { len, packed_chars }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        for ascii_char in self {
            write!(fmtr, "{}", ascii_char as char)?;
        }
        Ok(())
    }
}

impl PartialEq<[u8]> for Name {
    fn eq(&self, other: &[u8]) -> bool {
        self.len() == other.len()
            && self.ascii_chars().eq(other.iter().copied())
    }
}

impl<'a> PartialEq<&'a [u8]> for Name {
    fn eq(&self, other: &&'a [u8]) -> bool {
        self == *other
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self == other.as_bytes()
    }
}

impl<'a> PartialEq<&'a str> for Name {
    fn eq(&self, other: &&'a str) -> bool {
        self == *other
    }
}

impl PartialOrd<[u8]> for Name {
    fn partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
        Some(self.ascii_chars().cmp(other.iter().copied()))
    }
}

impl<'a> PartialOrd<&'a [u8]> for Name {
    fn partial_cmp(&self, other: &&'a [u8]) -> Option<Ordering> {
        self.partial_cmp(*other)
    }
}

impl PartialOrd<str> for Name {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        self.partial_cmp(other.as_bytes())
    }
}

impl<'a> PartialOrd<&'a str> for Name {
    fn partial_cmp(&self, other: &&'a str) -> Option<Ordering> {
        self.partial_cmp(*other)
    }
}

impl IntoIterator for Name {
    type Item = u8;
    type IntoIter = NameAsciiChars;

    fn into_iter(self) -> Self::IntoIter {
        self.ascii_chars()
    }
}

impl<'a> IntoIterator for &'a Name {
    type Item = u8;
    type IntoIter = NameAsciiChars;

    fn into_iter(self) -> Self::IntoIter {
        self.ascii_chars()
    }
}

impl serde::Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.bits)
    }
}

#[derive(Debug, Clone)]
struct NameDeVisitor;

impl<'de> serde::de::Visitor<'de> for NameDeVisitor {
    type Value = Name;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        formatter.write_str("64-bit unsigned integer in internal format")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let (len, packed_chars) = Name::unpack_parts(value);
        if len > Name::MAX_LEN as u64 || len < Name::MIN_LEN as u64 {
            Err(E::custom(format!("corrupted player name length {}", len)))
        } else {
            let mask = u64::MAX << (u64::BITS - Name::CHAR_BITS * len as u32);
            if packed_chars != packed_chars & mask {
                Err(E::custom(format!(
                    "corrupted player name characters {}",
                    packed_chars
                )))
            } else {
                Ok(Name { bits: value })
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_u64(NameDeVisitor)
    }
}

impl<'a> TryFrom<&'a [u8]> for Name {
    type Error = InvalidName;

    fn try_from(ascii_chars: &'a [u8]) -> Result<Self, Self::Error> {
        Self::new(ascii_chars)
    }
}

impl<'a> TryFrom<&'a str> for Name {
    type Error = InvalidName;

    fn try_from(ascii_str: &'a str) -> Result<Self, Self::Error> {
        Self::new(ascii_str.as_bytes())
    }
}

impl FromStr for Name {
    type Err = InvalidName;

    fn from_str(ascii_str: &str) -> Result<Self, Self::Err> {
        Self::try_from(ascii_str)
    }
}

#[derive(Debug, Clone)]
pub struct NameAsciiChars {
    len: u64,
    packed_chars: u64,
}

impl Iterator for NameAsciiChars {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let packed_char =
                self.packed_chars >> (u64::BITS - Name::CHAR_BITS);
            self.packed_chars <<= Name::CHAR_BITS;
            Some(Name::unpack_char(packed_char as u8))
        }
    }
}

impl DoubleEndedIterator for NameAsciiChars {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let shift_count =
                u64::BITS - Name::CHAR_BITS * (self.len as u32 + 1);
            let mask = (1 << Name::CHAR_BITS) - 1;
            let packed_char = (self.packed_chars >> shift_count) & mask;
            self.packed_chars &= !mask << shift_count;
            Some(Name::unpack_char(packed_char as u8))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionalName {
    bits: u64,
}

impl OptionalName {
    pub const NONE: Self = Self { bits: u64::MIN };

    pub const fn some(player_name: Name) -> Self {
        Self { bits: player_name.bits }
    }

    pub const fn from_option(option: Option<Name>) -> Self {
        match option {
            Some(player_name) => Self::some(player_name),
            None => Self::NONE,
        }
    }

    pub const fn into_option(self) -> Option<Name> {
        if self.bits == Self::NONE.bits {
            None
        } else {
            Some(Name { bits: self.bits })
        }
    }
}

impl Default for OptionalName {
    fn default() -> Self {
        Self::NONE
    }
}

impl From<Option<Name>> for OptionalName {
    fn from(value: Option<Name>) -> Self {
        Self::from_option(value)
    }
}

impl From<OptionalName> for Option<Name> {
    fn from(value: OptionalName) -> Self {
        value.into_option()
    }
}

impl From<Name> for OptionalName {
    fn from(value: Name) -> Self {
        Self::some(value)
    }
}

impl serde::Serialize for OptionalName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.bits)
    }
}

#[derive(Debug, Clone)]
struct OptionalNameDeVisitor;

impl<'de> serde::de::Visitor<'de> for OptionalNameDeVisitor {
    type Value = OptionalName;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        formatter.write_str("64-bit unsigned integer in internal format")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value == OptionalName::NONE.bits {
            Ok(OptionalName::NONE)
        } else {
            NameDeVisitor.visit_u64(value).map(OptionalName::some)
        }
    }
}

impl<'de> serde::Deserialize<'de> for OptionalName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_u64(OptionalNameDeVisitor)
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Player {
    pub name: Name,
    pub location: human::Location,
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use super::{InvalidName, Name, OptionalName};

    #[test]
    fn name_next() {
        let expected =
            [b'7', b'8', b'9', b'4', b'5', b'6', b'1', b'2', b'3', b'0'];
        let actual: Vec<_> =
            Name::new(b"7894561230").unwrap().ascii_chars().collect();
        assert_eq!(&actual[..], &expected[..]);
    }

    #[test]
    fn name_next_back() {
        let expected =
            [b'0', b'3', b'2', b'1', b'6', b'5', b'4', b'9', b'8', b'7'];
        let actual: Vec<_> =
            Name::new(b"7894561230").unwrap().ascii_chars().rev().collect();
        assert_eq!(&actual[..], &expected[..]);
    }

    #[test]
    fn valid_name_chars_only_digts_max() {
        let expected = b"7894561230";
        let actual = Name::new(b"7894561230").unwrap();
        assert_eq!(actual, &expected[..]);
    }

    #[test]
    fn valid_name_chars_only_upper_0_max() {
        let expected = b"CBAFDEIHGJ";
        let actual = Name::new(b"CBAFDEIHGJ").unwrap();
        assert_eq!(actual, &expected[..]);
    }

    #[test]
    fn valid_name_chars_only_upper_1_max() {
        let expected = b"MLKPONSRQT";
        let actual = Name::new(b"MLKPONSRQT").unwrap();
        assert_eq!(actual, &expected[..]);
    }

    #[test]
    fn valid_name_chars_only_upper_2() {
        let expected = b"WVUZYX";
        let actual = Name::new(b"WVUZYX").unwrap();
        assert_eq!(actual, &expected[..]);
    }

    #[test]
    fn valid_name_chars_only_lower_0_max() {
        let expected = b"cbafdeihgj";
        let actual = Name::new(b"cbafdeihgj").unwrap();
        assert_eq!(actual, &expected[..]);
    }

    #[test]
    fn valid_name_chars_only_lower_1_max() {
        let expected = b"mlkponsrqt";
        let actual = Name::new(b"mlkponsrqt").unwrap();
        assert_eq!(actual, &expected[..]);
    }

    #[test]
    fn valid_name_chars_only_lower_2() {
        let expected = b"wvuzyx";
        let actual = Name::new(b"wvuzyx").unwrap();
        assert_eq!(actual, &expected[..]);
    }

    #[test]
    fn valid_name_chars_only_special() {
        let expected = b"-_";
        let actual = Name::new(b"-_").unwrap();
        assert_eq!(actual, &expected[..]);
    }

    #[test]
    fn valid_name_chars_mixed() {
        let expected = b"Gamer_13";
        let actual = Name::new(b"Gamer_13").unwrap();
        assert_eq!(actual, &expected[..]);
    }

    #[test]
    fn valid_name_min() {
        let expected = b"a";
        let actual = Name::new(b"a").unwrap();
        assert_eq!(actual, &expected[..]);
    }

    #[test]
    fn invalid_name_too_short() {
        let expected = InvalidName::TooShort(0);
        let actual = Name::new(b"").unwrap_err();
        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_name_too_long() {
        let expected = InvalidName::TooLong(11);
        let actual = Name::new(b"12345678910").unwrap_err();
        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_name_too_long_bits_threshold() {
        let expected = InvalidName::TooLong(15);
        let actual = Name::new(b"123456789102345").unwrap_err();
        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_name_invalid_char_special() {
        let expected = InvalidName::InvalidChar(b'!');
        let actual = Name::new("fact!".as_bytes()).unwrap_err();
        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_name_invalid_char_control() {
        let expected = InvalidName::InvalidChar(b'\n');
        let actual = Name::new("fact\n".as_bytes()).unwrap_err();
        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_name_invalid_char_unicode() {
        let expected =
            InvalidName::InvalidChar(*"ç".as_bytes().last().unwrap());
        let actual = Name::new("façade".as_bytes()).unwrap_err();
        assert_eq!(actual, expected);
    }

    #[test]
    fn name_equals() {
        let left = Name::new(b"hi-world8").unwrap();
        let right = Name::try_from("hi-world8").unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn name_equals_cmp() {
        let left = Name::new(b"hi-world8").unwrap();
        let right = Name::try_from("hi-world8").unwrap();
        assert_eq!(left.cmp(&right), Ordering::Equal);
    }

    #[test]
    fn name_not_equals_beginning() {
        let left = Name::new(b"ai-world8").unwrap();
        let right = Name::try_from("hi-world8").unwrap();
        assert_ne!(left, right);
    }

    #[test]
    fn name_not_equals_middle() {
        let left = Name::new(b"hi_world8").unwrap();
        let right = Name::try_from("hi-world8").unwrap();
        assert_ne!(left, right);
    }

    #[test]
    fn name_not_equals_end() {
        let left = Name::new(b"hi-worldX").unwrap();
        let right = Name::try_from("hi-world8").unwrap();
        assert_ne!(left, right);
    }

    #[test]
    fn name_less_beginning() {
        let left = Name::new(b"ai-world8").unwrap();
        let right = Name::try_from("hi-world8").unwrap();
        assert_eq!(left.cmp(&right), Ordering::Less);
    }

    #[test]
    fn name_less_middle() {
        let left = Name::new(b"hi-world8").unwrap();
        let right = Name::try_from("hi_world8").unwrap();
        assert_eq!(left.cmp(&right), Ordering::Less);
    }

    #[test]
    fn name_less_end() {
        let left = Name::new(b"hi-worldX").unwrap();
        let right = Name::try_from("hi-world_").unwrap();
        assert_eq!(left.cmp(&right), Ordering::Less);
    }

    #[test]
    fn name_less_len() {
        let left = Name::new(b"hi-world").unwrap();
        let right = Name::try_from("hi-world-").unwrap();
        assert_eq!(left.cmp(&right), Ordering::Less);
    }

    #[test]
    fn name_greater_beginning() {
        let left = Name::new(b"hi-world8").unwrap();
        let right = Name::try_from("_i-world8").unwrap();
        assert_eq!(left.cmp(&right), Ordering::Greater);
    }

    #[test]
    fn name_greater_middle() {
        let left = Name::new(b"hi_world8").unwrap();
        let right = Name::try_from("hiYworld8").unwrap();
        assert_eq!(left.cmp(&right), Ordering::Greater);
    }

    #[test]
    fn name_greater_end() {
        let left = Name::new(b"hi-world8").unwrap();
        let right = Name::try_from("hi-world0").unwrap();
        assert_eq!(left.cmp(&right), Ordering::Greater);
    }

    #[test]
    fn name_greater_len() {
        let left = Name::new(b"hi-world-").unwrap();
        let right = Name::try_from("hi-world").unwrap();
        assert_eq!(left.cmp(&right), Ordering::Greater);
    }

    #[test]
    fn name_ser_de_preserve() {
        let data = Name::new(b"blober").unwrap();
        let encoded = bincode::serialize(&data).unwrap();
        let decoded: Name = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn optional_name_some_into_preserve() {
        let name = Name::new(b"-rf").unwrap();
        let optional = OptionalName::some(name);
        let converted = optional.into_option().unwrap();
        assert_eq!(name, converted);
    }

    #[test]
    fn optional_name_none() {
        let optional = OptionalName::NONE;
        assert!(optional.into_option().is_none());
    }

    #[test]
    fn optional_name_equals_some() {
        let left = OptionalName::some(Name::new(b"foo_bar").unwrap());
        let right = OptionalName::some(Name::try_from("foo_bar").unwrap());
        assert_eq!(left, right);
    }

    #[test]
    fn optional_name_equals_none() {
        let left = OptionalName::NONE;
        let right = OptionalName::NONE;
        assert_eq!(left, right);
    }

    #[test]
    fn optional_name_equals_some_cmp() {
        let left = OptionalName::some(Name::new(b"foo_bar").unwrap());
        let right = OptionalName::some(Name::try_from("foo_bar").unwrap());
        assert_eq!(left.cmp(&right), Ordering::Equal);
    }

    #[test]
    fn optional_name_equals_none_cmp() {
        let left = OptionalName::NONE;
        let right = OptionalName::NONE;
        assert_eq!(left.cmp(&right), Ordering::Equal);
    }

    #[test]
    fn optional_name_not_equals_some_some() {
        let left = OptionalName::some(Name::new(b"foo_bar").unwrap());
        let right = OptionalName::some(Name::try_from("f0o_bar").unwrap());
        assert_ne!(left, right);
    }

    #[test]
    fn optional_name_not_equals_none_some() {
        let left = OptionalName::NONE;
        let right = OptionalName::some(Name::try_from("f0o_bar").unwrap());
        assert_ne!(left, right);
    }

    #[test]
    fn optional_name_not_equals_some_none() {
        let left = OptionalName::some(Name::new(b"foo_bar").unwrap());
        let right = OptionalName::NONE;
        assert_ne!(left, right);
    }

    #[test]
    fn optional_name_less_some_some() {
        let left = OptionalName::some(Name::new(b"fo0_bar").unwrap());
        let right = OptionalName::some(Name::try_from("foo_bar").unwrap());
        assert_eq!(left.cmp(&right), Ordering::Less);
    }

    #[test]
    fn optional_name_less_none_some() {
        let left = OptionalName::NONE;
        let right = OptionalName::some(Name::try_from("foo_bar").unwrap());
        assert_eq!(left.cmp(&right), Ordering::Less);
    }

    #[test]
    fn optional_name_less_none_some_min() {
        let left = OptionalName::NONE;
        let right = OptionalName::some(Name::try_from("-").unwrap());
        assert_eq!(left.cmp(&right), Ordering::Less);
    }

    #[test]
    fn optional_name_greater_some_some() {
        let left = OptionalName::some(Name::new(b"yoo_bar").unwrap());
        let right = OptionalName::some(Name::try_from("Yoo_bar").unwrap());
        assert_eq!(left.cmp(&right), Ordering::Greater);
    }

    #[test]
    fn optional_name_gerater_some_none() {
        let left = OptionalName::some(Name::try_from("baz").unwrap());
        let right = OptionalName::NONE;
        assert_eq!(left.cmp(&right), Ordering::Greater);
    }

    #[test]
    fn optional_name_gerater_some_min_none() {
        let left = OptionalName::some(Name::try_from("-").unwrap());
        let right = OptionalName::NONE;
        assert_eq!(left.cmp(&right), Ordering::Greater);
    }

    #[test]
    fn optional_name_ser_de_preserve_some() {
        let data = OptionalName::some(Name::new(b"blober").unwrap());
        let encoded = bincode::serialize(&data).unwrap();
        let decoded: OptionalName = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn optional_name_ser_de_preserve_none() {
        let data = OptionalName::NONE;
        let encoded = bincode::serialize(&data).unwrap();
        let decoded: OptionalName = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(data, decoded);
    }
}
