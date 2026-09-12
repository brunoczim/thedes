use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize, de, ser};
pub use thedes_db_core::rid::raw;

pub use raw::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rid {
    bits: raw::Bits,
}

impl Rid {
    pub fn new(encoded: impl AsRef<[u8]>) -> Result<Self, Error> {
        let encoded = raw::encode(encoded.as_ref())?;
        Ok(Self::unstable_from_raw(encoded))
    }

    pub fn unstable_from_raw(raw: raw::Bits) -> Self {
        Self { bits: raw }
    }

    pub fn bits(&self) -> raw::Bits {
        self.bits
    }
}

impl fmt::Display for Rid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0; raw::ENCODED_SIZE];
        let trimmed = raw::decode(self.bits(), &mut buf);
        for byte in trimmed {
            write!(f, "{}", *byte as char)?;
        }
        Ok(())
    }
}

impl FromStr for Rid {
    type Err = raw::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::new(input)
    }
}

impl TryFrom<&'_ [u8]> for Rid {
    type Error = Error;

    fn try_from(value: &'_ [u8]) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&'_ str> for Rid {
    type Error = Error;

    fn try_from(value: &'_ str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Serialize for Rid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let mut buf = [0; raw::ENCODED_SIZE];
            let trimmed = raw::decode(self.bits(), &mut buf);
            let as_str = str::from_utf8(trimmed)
                .map_err(<S::Error as ser::Error>::custom)?;
            as_str.serialize(serializer)
        } else {
            self.bits().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Rid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RidVisitor;

        impl<'de> de::Visitor<'de> for RidVisitor {
            type Value = Rid;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "Thedes DB resouce identifier")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                v.parse().map_err(E::custom)
            }

            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let mut buf = [0; raw::ENCODED_SIZE];
                let trimmed = raw::decode(v, &mut buf);
                Rid::new(trimmed).map_err(E::custom)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(RidVisitor)
        } else {
            deserializer.deserialize_u128(RidVisitor)
        }
    }
}

#[cfg(test)]
mod test {
    use crate as thedes_db;
    use crate::{rid, rid::Rid};

    fn prop_encode_decode_human(original: Rid) {
        let encoded = serde_json::to_vec(&original).unwrap();
        let decoded = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    fn prop_encode_decode_bin(original: Rid) {
        let encoded = rmp_serde::to_vec(&original).unwrap();
        let decoded = rmp_serde::from_slice(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn macro_should_produce_equal() {
        let rid_a = rid!(th0/t:player);
        let rid_b = rid!(th0/t:player);

        assert_eq!(rid_a, rid_b);
    }

    #[test]
    fn macro_should_produce_different() {
        let rid_a = rid!(th0/t:player);
        let rid_b = rid!(th0/m:world);

        assert_ne!(rid_a, rid_b);
    }

    #[test]
    fn macro_should_display_correct() {
        let my_rid = rid!(th0/t:player);
        let stringified = my_rid.to_string();
        assert_eq!(stringified, "th0/t:player");
    }

    #[test]
    fn macro_should_display_with_hash_in_the_middle() {
        let my_rid = rid!(th0/t:pl%ayer);
        let stringified = my_rid.to_string();
        assert_eq!(stringified, "th0/t:pl%ayer");
    }

    #[test]
    fn macro_should_display_with_hash_trimmed() {
        let my_rid = rid!(th0/t:pl%ayer%%%);
        let stringified = my_rid.to_string();
        assert_eq!(stringified, "th0/t:pl%ayer");
    }

    #[test]
    fn macro_should_display_with_underscore() {
        let my_rid = rid!(th0/t:pl_ayer);
        let stringified = my_rid.to_string();
        assert_eq!(stringified, "th0/t:pl_ayer");
    }

    #[test]
    fn encode_decode_players_human() {
        prop_encode_decode_human(rid!(th0/t:players));
    }

    #[test]
    fn encode_decode_players_bin() {
        prop_encode_decode_bin(rid!(th0/t:players));
    }

    #[test]
    fn encode_decode_random_human() {
        prop_encode_decode_human(rid!(th0/o:3yrkihv%u%x_bzt5akm7));
    }

    #[test]
    fn encode_decode_random_bin() {
        prop_encode_decode_bin(rid!(th0/o:3yrkihv%u%x_bzt5akm7));
    }
}
