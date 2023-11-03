use gardiz::{coord::Vec2, direc::Direction, rect::Rect};
use ndarray::{Array, Ix2};
use num::CheckedSub;
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt,
    ops::{Index, IndexMut},
    str::{self, FromStr},
};
use thiserror::Error;

pub type Coord = u16;

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
pub enum InvalidPlayerName {
    #[error(
        "Given player name is too short, minimum is {}, found {}",
        PlayerName::MIN_LEN,
        .0,
    )]
    TooShort(usize),
    #[error(
        "Given player name is too long, maximum is {}, found {}",
        PlayerName::MAX_LEN,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerName {
    bits: u64,
}

impl PlayerName {
    pub const MIN_LEN: usize = 1;

    pub const MAX_LEN: usize = 10;

    const CHAR_BITS: u32 = 6;

    const CHARS_BITS: u32 = Self::CHAR_BITS * Self::MAX_LEN as u32;

    const HYPHEN_OFFSET: u8 = 0;

    const DIGITS_OFFSET: u8 = Self::HYPHEN_OFFSET + 1;

    const UPPER_OFFSET: u8 = Self::DIGITS_OFFSET + 10;

    const LOWER_OFFSET: u8 = Self::UPPER_OFFSET + 26;

    const UNDERSCORE_OFFSET: u8 = Self::LOWER_OFFSET + 26;

    pub const MIN: Self = Self { bits: Self::pack_parts(0, 0) };

    pub const MAX: Self =
        Self { bits: Self::pack_parts(Self::MAX_LEN as u64, u64::MAX) };

    const fn pack_parts(len: u64, packed_chars: u64) -> u64 {
        let shifted_len = (len - Self::MIN_LEN as u64) << Self::CHARS_BITS;
        let masked_chars =
            packed_chars & (u64::MAX >> (u64::BITS - Self::CHARS_BITS));
        shifted_len | masked_chars
    }

    const fn unpack_parts(packed: u64) -> (u64, u64) {
        let len =
            (packed >> Self::CHARS_BITS).saturating_add(Self::MIN_LEN as u64);
        let packed_chars =
            packed & (u64::MAX >> (u64::BITS - Self::CHARS_BITS));
        (len, packed_chars)
    }

    const fn pack_char(ascii_char: u8) -> Result<u8, InvalidPlayerName> {
        if ascii_char == b'-' {
            Ok(Self::HYPHEN_OFFSET)
        } else if ascii_char >= b'0' && ascii_char <= b'9' {
            Ok(ascii_char - b'0' + Self::DIGITS_OFFSET)
        } else if ascii_char >= b'A' && ascii_char <= b'Z' {
            Ok(ascii_char - b'A' + Self::UPPER_OFFSET)
        } else if ascii_char >= b'a' && ascii_char <= b'z' {
            Ok(ascii_char - b'a' + Self::LOWER_OFFSET)
        } else if ascii_char == b'_' {
            Ok(Self::UNDERSCORE_OFFSET)
        } else {
            Err(InvalidPlayerName::InvalidChar(ascii_char))
        }
    }

    const fn unpack_char(packed_char: u8) -> u8 {
        if packed_char == Self::UNDERSCORE_OFFSET {
            b'_'
        } else if packed_char >= Self::LOWER_OFFSET {
            packed_char - Self::LOWER_OFFSET + b'a'
        } else if packed_char >= Self::UPPER_OFFSET {
            packed_char - Self::UPPER_OFFSET + b'A'
        } else if packed_char >= Self::DIGITS_OFFSET {
            packed_char - Self::DIGITS_OFFSET + b'0'
        } else {
            b'-'
        }
    }

    pub const fn new(ascii_chars: &[u8]) -> Result<Self, InvalidPlayerName> {
        if ascii_chars.len() < Self::MIN_LEN {
            return Err(InvalidPlayerName::TooShort(ascii_chars.len()));
        }

        if ascii_chars.len() > Self::MAX_LEN {
            return Err(InvalidPlayerName::TooLong(ascii_chars.len()));
        }

        let mut packed_chars = 0;
        let mut i = ascii_chars.len();

        while i > 0 {
            i -= 1;
            packed_chars <<= Self::CHAR_BITS;
            packed_chars |= match Self::pack_char(ascii_chars[i]) {
                Ok(packed) => packed as u64,
                Err(error) => return Err(error),
            }
        }

        let bits = Self::pack_parts(ascii_chars.len() as u64, packed_chars);

        Ok(Self { bits })
    }

    pub const fn ascii_chars(self) -> PlayerNameAsciiChars {
        let (len, packed_chars) = Self::unpack_parts(self.bits);
        PlayerNameAsciiChars { len, packed_chars }
    }
}

impl fmt::Display for PlayerName {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        for ascii_char in self {
            write!(fmtr, "{}", ascii_char as char)?;
        }
        Ok(())
    }
}

impl PartialOrd for PlayerName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PlayerName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ascii_chars().cmp(other.ascii_chars())
    }
}

impl IntoIterator for PlayerName {
    type Item = u8;
    type IntoIter = PlayerNameAsciiChars;

    fn into_iter(self) -> Self::IntoIter {
        self.ascii_chars()
    }
}

impl<'a> IntoIterator for &'a PlayerName {
    type Item = u8;
    type IntoIter = PlayerNameAsciiChars;

    fn into_iter(self) -> Self::IntoIter {
        self.ascii_chars()
    }
}

impl serde::Serialize for PlayerName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.bits)
    }
}

#[derive(Debug, Clone)]
struct PlayerNameDeVisitor;

impl<'de> serde::de::Visitor<'de> for PlayerNameDeVisitor {
    type Value = PlayerName;

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
        let (len, packed_chars) = PlayerName::unpack_parts(value);
        if len > PlayerName::MAX_LEN as u64 {
            Err(E::custom(format!("corrupted player name length {}", len)))
        } else if (1 << (6 * len)) - 1 < packed_chars {
            Err(E::custom(format!(
                "corrupted player name characters {}",
                packed_chars
            )))
        } else {
            Ok(PlayerName { bits: value })
        }
    }
}

impl<'de> serde::Deserialize<'de> for PlayerName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_u64(PlayerNameDeVisitor)
    }
}

impl<'a> TryFrom<&'a [u8]> for PlayerName {
    type Error = InvalidPlayerName;

    fn try_from(ascii_chars: &'a [u8]) -> Result<Self, Self::Error> {
        Self::new(ascii_chars)
    }
}

impl<'a> TryFrom<&'a str> for PlayerName {
    type Error = InvalidPlayerName;

    fn try_from(ascii_str: &'a str) -> Result<Self, Self::Error> {
        Self::new(ascii_str.as_bytes())
    }
}

impl FromStr for PlayerName {
    type Err = InvalidPlayerName;

    fn from_str(ascii_str: &str) -> Result<Self, Self::Err> {
        Self::try_from(ascii_str)
    }
}

#[derive(Debug, Clone)]
pub struct PlayerNameAsciiChars {
    len: u64,
    packed_chars: u64,
}

impl Iterator for PlayerNameAsciiChars {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let packed_char =
                self.packed_chars & ((1 << PlayerName::CHAR_BITS) - 1);
            self.packed_chars >>= PlayerName::CHAR_BITS;
            Some(PlayerName::unpack_char(packed_char as u8))
        }
    }
}

impl DoubleEndedIterator for PlayerNameAsciiChars {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let shift_count = PlayerName::CHAR_BITS * (self.len as u32 - 1);
            let mask = (1 << shift_count) - 1;
            let packed_char = self.packed_chars & mask;
            self.packed_chars &= !mask;
            Some(PlayerName::unpack_char(packed_char as u8))
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
    serde::Serialize,
    serde::Deserialize,
)]
pub struct HumanLocation {
    pub head: Vec2<Coord>,
    pub facing: Direction,
}

impl HumanLocation {
    pub fn pointer(self) -> Vec2<Coord> {
        self.head.move_one(self.facing)
    }

    pub fn checked_pointer(self) -> Option<Vec2<Coord>> {
        self.head.checked_move(self.facing)
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
    pub name: PlayerName,
    pub location: HumanLocation,
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
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Ground {
    Grass,
    Sand,
}

impl Default for Ground {
    fn default() -> Self {
        Self::Grass
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
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Biome {
    Plains,
    Desert,
}

impl Default for Biome {
    fn default() -> Self {
        Self::Plains
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
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct MapCell {
    pub player: Option<PlayerName>,
    pub ground: Ground,
    pub biome: Biome,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct MapSlice {
    offset: Vec2<Coord>,
    matrix: Array<MapCell, Ix2>,
}

impl MapSlice {
    pub fn generate<F>(view: Rect<Coord>, mut generator: F) -> Self
    where
        F: FnMut(Vec2<Coord>) -> MapCell,
    {
        Self {
            offset: view.start,
            matrix: Array::from_shape_fn(
                [usize::from(view.size.y), usize::from(view.size.x)],
                |(y, x)| {
                    generator(Vec2 {
                        x: x as Coord + view.start.x,
                        y: y as Coord + view.start.y,
                    })
                },
            ),
        }
    }

    pub fn default(view: Rect<Coord>) -> Self {
        Self {
            offset: view.start,
            matrix: Array::default([
                usize::from(view.size.y),
                usize::from(view.size.x),
            ]),
        }
    }

    pub fn view(&self) -> Rect<Coord> {
        Rect {
            start: self.offset,
            size: Vec2 {
                y: self.matrix.dim().0 as Coord,
                x: self.matrix.dim().1 as Coord,
            },
        }
    }

    pub fn sub(&self, view: Rect<Coord>) -> Option<Self> {
        if self.view().has_point(view.end_inclusive())
            && self.view().has_point(view.start)
        {
            Some(Self::generate(view, |point| self[point].clone()))
        } else {
            None
        }
    }

    pub fn get(&self, index: Vec2<Coord>) -> Option<&MapCell> {
        let shifted = index.checked_sub(&self.offset)?;
        self.matrix.get([usize::from(shifted.y), usize::from(shifted.x)])
    }

    pub fn get_mut(&mut self, index: Vec2<Coord>) -> Option<&mut MapCell> {
        let shifted = index.checked_sub(&self.offset)?;
        self.matrix.get_mut([usize::from(shifted.y), usize::from(shifted.x)])
    }
}

impl Index<Vec2<Coord>> for MapSlice {
    type Output = MapCell;

    fn index(&self, index: Vec2<Coord>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IndexMut<Vec2<Coord>> for MapSlice {
    fn index_mut(&mut self, index: Vec2<Coord>) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct GameSnapshot {
    pub map: MapSlice,
    pub players: BTreeMap<PlayerName, Player>,
}
