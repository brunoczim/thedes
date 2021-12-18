use crate::error::{BadSeedString, Error, Result};
use std::{fmt, str::FromStr};

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
pub struct Seed {
    pub bits: u64,
}

impl FromStr for Seed {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        let bits =
            u64::from_str_radix(string, 16).map_err(|_| BadSeedString)?;
        Ok(Seed { bits })
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{:x}", self.bits)
    }
}
